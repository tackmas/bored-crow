//! This module exclusively uses the `W`-suffixed (wide) variants of Windows API
//! functions (e.g. `CreateFileW`, `GetModuleFileNameW`). As a result, all string
//! buffers exchanged with the OS in this module are UTF-16 and can be assumed
//! to decode validly

use std::io::Write;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, OpenOptions};
use std::path::{Path, PathBuf};
use std::ptr::{null_mut};

use bytes::Bytes;

use iced::widget::image::Handle;

use image::imageops::{self, FilterType};

use win32_version_info::VersionInfo;

use windows::core::{BOOL, Error as WinError, HSTRING, Interface, PCWSTR, PWSTR, w, Result as WinResult};
use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize,
    IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_FORMAT , PROCESS_QUERY_LIMITED_INFORMATION, 
    QueryFullProcessImageNameW 
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_SHORTPATH};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GA_ROOTOWNER, GetAncestor, GetLastActivePopup, GetWindowLongW,
    GetWindowTextLengthW, GetWindowThreadProcessId, 
    GWL_EXSTYLE, IsWindowVisible, WS_EX_TOOLWINDOW
};

use windows_icons::IconSize;

use super::{App, IncorporableApp, IncorporableApps};

pub fn list_installed_apps() -> Vec<IncorporableApp> {
    let guessed_average_app_count = 30;
    let mut incorporable_apps = Vec::with_capacity(guessed_average_app_count);

    fn recursive(path_buf: &mut PathBuf, apps: &mut Vec<IncorporableApp>) {
        let dir_entries = match fs::read_dir(&path_buf) {
            Ok(dir_entries) => dir_entries,
            Err(err) => {
                eprintln!("Error reading directory: {:?}; Err: {}", path_buf, err);
                return;
            } 
        };

        let guessed_max_component_path_len = 30;
        path_buf.reserve(guessed_max_component_path_len);

        for entry in dir_entries.filter_map(Result::ok) {
            let file_name = entry.file_name();

            path_buf.push(&file_name);

            let extension = {
                let file_name: &Path = file_name.as_ref();
                file_name.extension()
            };

            if extension == Some(OsStr::new("lnk")) {
                let lnk_name = {
                    let as_path: &Path = file_name.as_ref();

                    as_path.file_prefix().expect(
                        "A prefix always exists since it is derived from a valid, existing entry"
                    )
                };

                handle_lnk(path_buf, &lnk_name, apps);
            } else if is_relevant_dir(entry, &file_name) {
                recursive(path_buf, apps);
            }

            path_buf.pop();
        }
    }

    // COM must be initialized on the calling thread. Otherwise the function will not work
    fn handle_lnk(path_to_lnk: &Path, lnk_name: &OsStr, incorporable_apps: &mut Vec<IncorporableApp>) {
        unsafe {
            let exe_path = {
                // Create the ShellLink COM object, ask for the IShellLinkW interface.
                let Ok(shell_link): WinResult<IShellLinkW> = 
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) else { return; };

                // Load the .lnk file's bytes via IPersistFile.
                let Ok(persist_file): WinResult<IPersistFile> = 
                    shell_link.cast() else { return; };

                let wide_path = HSTRING::from(path_to_lnk);
                if let Err(err) = persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM_READ) {
                    eprintln!("Error loading into IPersistFile: {err}");
                    return;
                }

                // Resolve the shell link in case is has been renamed/moved
                let no_dialog_window = HWND(null_mut());
                if let Err(err) = shell_link.Resolve(no_dialog_window, 0) {
                    eprintln!("Error resolving shell link: {err}");
                    return;
                }

                let mut exe_path_buf = [0u16; MAX_PATH as usize];
                let no_additional_data = null_mut();
                let default_path_info_flag = 0;
                if let Err(err) = shell_link.GetPath(&mut exe_path_buf, no_additional_data, default_path_info_flag) {
                    eprintln!("Error getting path: {err}");
                    return;
                }
                
                // Find the length of the path (UTF-16)
                let path_len = exe_path_buf.iter()
                    .position(|&c| c == 0)
                    .unwrap_or(exe_path_buf.len());

                let exe_extension = w!(".exe");
                let Some(extension_idx) = path_len.checked_sub(exe_extension.len()) else {
                    return;
                };

                if !exe_path_buf[0..path_len].ends_with(w!(".exe").as_wide()) 
                || exe_path_already_exists(incorporable_apps, &exe_path_buf, path_len) {
                    return;
                }

                String::from_utf16_lossy(&exe_path_buf[..path_len])
            };

            // FIXME: return default icon if Err. also optimzation maybe available; this function
            // is expensive as hell
            let icon = match fetch_icon_in_bytes2(exe_path.as_ref()) {
                Ok(icon_in_bytes) => icon_in_bytes,
                Err(err) => {
                    eprintln!("Error getting icon from lnk;\nPath to lnk: {path_to_lnk:?};\nErr: {err:?}");

                    return;
                }               
            };

            let name = lnk_name
                .to_string_lossy()
                .to_string();

            let app = App::from(name, exe_path, icon);
            let incorporable_app = IncorporableApp::new_with(app);

            incorporable_apps.push(incorporable_app);
        }
    }    

    let mut root = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs");

    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap(); }

    recursive(&mut root, &mut incorporable_apps);

    unsafe { CoUninitialize(); }

    incorporable_apps.sort_by(|a, b| desktop::ordering_by_alphabetical(&a.inner.name, &b.inner.name));

    incorporable_apps
}

fn is_relevant_dir(dir_entry: DirEntry, file_name: &OsStr) -> bool {
    match dir_entry.file_type() {
        Ok(file_type) => {
            file_type.is_dir() && file_name != OsStr::new("Windows Kits") 
        },
        Err(err) => {
            eprintln!("Error getting file type of dir entry: {:?}; Err: {}", dir_entry, err);
            
            false
        }
    }
}

fn fetch_icon_in_bytes(path_to_icon: &str, name: &str) -> Result<Bytes, systemicons::Error> {
    systemicons::get_icon(path_to_icon, 32)
        .map(|bytes| {
            if name == "7-Zip File Manager" {
                OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open("C:\\ttttdebug.png")
                    .unwrap()
                    .write_all(&bytes)
                    .unwrap();                
            }

            Bytes::from(bytes)
        })
}

fn fetch_icon_in_bytes2(path_to_icon: &Path) -> Result<Handle, Box<dyn Error>> {
    windows_icons::get_icon_by_path_with_size(path_to_icon, IconSize::Small)
        .map(|icon| {
            let (width, height) = icon.dimensions();
            let bytes = Bytes::from(icon.into_vec());
            Handle::from_rgba(width, height, bytes)
        })
}

// This function return None if `path_to_file` resolves to a invalid name (`.` & `..`),
// otherwise Some(..)
fn get_file_elegant_name(path_to_file: &Path) -> Option<String> {
    match VersionInfo::from_file(path_to_file) {
        Ok(version_info) => {
            if version_info.file_description.is_empty() {
                file_name_from_path(path_to_file)
            } else {
                Some(version_info.file_description)
            }
        },
        Err(_) => file_name_from_path(path_to_file)
    }
}

fn file_name_from_path(path_to_file: &Path) -> Option<String> {
    path_to_file
        .file_name()
        .map(|path_to_file| {
            path_to_file.to_string_lossy().to_string()
        })
}

pub fn list_running_apps(mut incorporable_apps: IncorporableApps) -> IncorporableApps {
    // SAFETY: `lparam` is &mut IncorporableApps casted to LPARAM according to `enum_windows_proc`
    // safety requirement. The mutable reference to `incorporable_apps` is not alive during the
    // call to EnumWindows, so no two mutable references are alive at the same time.
    let res = unsafe { 
        let lparam = LPARAM(&mut incorporable_apps as *mut _ as isize);

        EnumWindows(Some(enum_windows_proc), lparam)
    };

    incorporable_apps
}

// SAFETY: `lparam` must be a valid &mut IncorporableApps casted to a LPARAM 
// according to the following process:
// &mut IncorporableApps -> *mut IncorporableApps -> isize -> LPARAM(isize)
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let skip = BOOL(1);

    if !is_window_visible(hwnd) {
        return skip;
    }

     // SAFETY: `lparam` must be a valid &mut IncorporableApps casted to a LPARAM as remarked
    let incorporable_apps = unsafe { 
        &mut *(lparam.0 as *mut IncorporableApps) 
    };

    let exe_path = unsafe {
        let mut pid = 0u32;
        let is_err = GetWindowThreadProcessId(hwnd, Some(&mut pid)) == 0;

        if is_err {
            eprintln!("Error getting PID: {}", WinError::from_thread());
            return skip;
        }

        let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(process_handle) => process_handle,
            Err(err) => {
                eprintln!("Error getting process handle: {err}");
                return skip;                
            }
        };       

        let mut exe_path_buf = [0u16; MAX_PATH as usize];
        let mut path_len = exe_path_buf.len() as u32;

        let res = {
            let win32_path_format = PROCESS_NAME_FORMAT(0);

            QueryFullProcessImageNameW(
                process_handle, win32_path_format, PWSTR(exe_path_buf.as_mut_ptr()), &mut path_len
            )
        };
            
        if let Err(err) = res {
            eprintln!("Error querying process image name: {err}");
            return skip
        }

        if let Err(err) = CloseHandle(process_handle) {
            eprintln!("Error closing process handle: {err}");
            return skip;
        }
        if exe_path_already_exists(incorporable_apps.running_apps(), &exe_path_buf, path_len as usize) {
            return skip;
        }

        String::from_utf16_lossy(&exe_path_buf[..path_len as usize])
    };
    let name = get_file_elegant_name(exe_path.as_ref())
        .expect("`exe_path` resolves to a executable file");

    let icon = match fetch_icon_in_bytes2(exe_path.as_ref()) {
        Ok(icon_in_bytes) => icon_in_bytes,
        Err(err) => {
            eprintln!("Error getting icon from lnk;\nPath to lnk: {exe_path:?};\nErr: {err:?}");

            return skip;
        }               
    };


    let app = App::from(name, exe_path, icon);
    let incorporable_app = IncorporableApp::new_with(app);

    incorporable_apps.inner.push(incorporable_app);

    skip
}

fn exe_path_already_exists(
    incorporable_apps: &[IncorporableApp], exe_path_buf: &[u16], path_len: usize
) -> bool {
    let reveresed_exe_path_as_chars = {
        let exe_path_reverse_iter = exe_path_buf[0..path_len]
            .iter().rev().copied();

        char::decode_utf16(exe_path_reverse_iter)
            .map(|decode_utf16_res| decode_utf16_res
                .expect(
                    "`exe_path_buf` should be valid UTF-16 per module docs (W-suffixed WinAPI only)"
                )
            )
    };

    incorporable_apps
        .iter()
        .any(|incorporable_app| incorporable_app.inner
            .exe_path
            .chars()
            .rev()
            .eq(reveresed_exe_path_as_chars.clone())
        )   
}

fn is_window_visible(hwnd: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() 
        || is_cloaked(hwnd).unwrap_or(true)
        || GetWindowTextLengthW(hwnd) == 0
        {
            return false;
        }

        // source for the rest of the filtering:
        // https://stackoverflow.com/questions/7277366/why-does-enumwindows-return-more-windows-than-i-expected
        let mut hwnd_old = HWND(null_mut());
        let mut hwnd_try = GetAncestor(hwnd, GA_ROOTOWNER);

        while hwnd_try != hwnd_old {
            hwnd_old = hwnd_try;
            hwnd_try = GetLastActivePopup(hwnd_old);

            if IsWindowVisible(hwnd_try).as_bool() {
                if hwnd_old != hwnd {
                    return false;
                } else {
                    break;
                }
            }
        }

        if GetWindowLongW(hwnd, GWL_EXSTYLE) & WS_EX_TOOLWINDOW.0 as i32 != 0 {
            return false;
        }
    }
    true
}

fn is_cloaked(hwnd: HWND) -> WinResult<bool> {
    let mut cloaked: u32 = 0;
    unsafe {
        let res = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        
        res.map(|_| cloaked != 0)
    }    
}

/*
    fn if_lnk(lnk_name: &OsStr, abs_path_to_lnk: &Path, apps: &mut Vec<App>) {
        let lnk = match Lnk::try_from(abs_path_to_lnk) {
            Ok(lnk) => lnk,
            Err(err) => {
                eprintln!("Error getting lnk from path;\nPath: {:?};\nErr: {},", abs_path_to_lnk, err);

                return;               
            }
        };

        let exe_path = match lnk.link_info.local_base_path
            .or(lnk.link_info.local_base_path_unicode) 
            .or_else(|| lnk.string_data.relative_path.map(|relative_path_to_target| {
                println!("{:?}", lnk.header.link_flags.contains(LinkFlags::HAS_LINK_TARGET_ID_LIST));

                let abs_path_to_target = 
                    lnk_target_relative_to_absolute_path(abs_path_to_lnk, &relative_path_to_target);

                abs_path_to_target.to_string_lossy().to_string()
            }))
        {
            Some(exe_path) if !exe_path.starts_with(r"C:\Windows") => exe_path,
            Some(_) => return,
            None => {
                eprintln!("No exe path found for lnk (Path to lnk: {:?})", abs_path_to_lnk);

                return;               
            }
        };

        let name = lnk_name.to_string_lossy();

        // FIXME: return default icon if Err
        let icon = match systemicons::get_icon(&abs_path_to_lnk.to_string_lossy(), 32)
            .map(Bytes::from) 
        {
            Ok(icon) => icon,
            Err(err) => {
                eprintln!("Error getting icon from lnk;\nPath to lnk: {abs_path_to_lnk:?};\nErr: {err:?}");

                return;
            }
        };
        let app = App::from(name.to_string(), exe_path, icon);
        
        apps.push(app);
    } 

fn lnk_target_relative_to_absolute_path(abs_path_to_lnk: &Path, relative_path_to_target: &Path) -> PathBuf {
    match abs_path_to_lnk
        .parent()
        .expect(".lnk is a file, therefore it must always have a parent directory")
        .join(relative_path_to_target)
        .canonicalize() 
    {
        Ok(abs_path_to_target) => abs_path_to_target,
        Err(err) => {
            panic!(
                "Error canonicalizing path; \
                \nErr: {err}; \
                \nPath to lnk: {abs_path_to_lnk:?}; \
                \nRelative path to target: {relative_path_to_target:?}"
            );
        }
    }
}

 fn get_system_code_page() -> Result<Encoding, ()> {
    match unsafe { GetACP() } {
        874 => Ok(WINDOWS_874),
        1200 => Ok(UTF_16LE),
        1250 => Ok(WINDOWS_1250),
        1251 => Ok(WINDOWS_1251),
        1252 => Ok(WINDOWS_1252),
        1253 => Ok(WINDOWS_1253),
        1254 => Ok(WINDOWS_1254),
        1255 => Ok(WINDOWS_1255),
        1256 => Ok(WINDOWS_1256),
        1257 => Ok(WINDOWS_1257),
        1258 => Ok(WINDOWS_1258),
        _ => Err(())
    }
}   
            // FIXME: return default icon if Err. also optimzation maybe available; this function
            // is expensive as hell
            let icon_in_bytes = match fetch_icon_in_bytes(&exe_path) {
                Ok(icon_in_bytes) => icon_in_bytes,
                Err(err) => {
                    eprintln!("Error getting icon from lnk;\nPath to lnk: {path_to_lnk:?};\nErr: {err:?}");

                    return;
                }               
            };
*/

