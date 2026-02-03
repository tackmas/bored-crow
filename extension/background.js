const blocked_sites = ["tiktok.com"];
const blockedHTMLURL = chrome.runtime.getURL("pages/blocked/blocked.html");

async function checkTab(tabId) {
  if (!tabId) return;

  const tab = await chrome.tabs.get(tabId);  
  console.assert(tab, "'tab' is undefined");

  const tabUrl = (tab.url || tab.pendingUrl).toString();

  console.log(tabId);

  if (blocked_sites.some(site => tabUrl.includes(site))) {
    updateTab(tabId, {url: blockedHTMLURL}, tabUrl);
  }
}

function pageReady() {
  return new Promise((resolve) => {
    chrome.runtime.onMessage.addListener(function listener({message}, sender, sendResponse) {
      if (message == "PAGE_LOADED" && sender.url == blockedHTMLURL) {
        resolve(sendResponse);

        chrome.runtime.onMessage.removeListener();

        return true;
      }
    });
  })
}

async function updateTab(tabId, url, tabUrl) {
  pageReady().then(sendResponse => {
    sendURL(sendResponse, tabUrl);
  });

  chrome.tabs.update(tabId, url);
}

function sendURL(sendResponse, tabUrl) {
  sendResponse(tabUrl);
}


// Events

chrome.tabs.onUpdated.addListener((tabId, changeInfo, _) => {
  if (changeInfo.status === "complete") {
    checkTab(tabId);
  }
});

chrome.tabs.onActivated.addListener((activeInfo) => {
    checkTab(activeInfo.tabId);
});
