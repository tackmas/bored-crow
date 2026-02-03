document.addEventListener("DOMContentLoaded", async (event) => {
  const blockedURL = await chrome.runtime.sendMessage({message: "PAGE_LOADED"});

  console.log(blockedURL);

  const elemURL = document.getElementById("site_url");

  elemURL.textContent = blockedURL;
});

