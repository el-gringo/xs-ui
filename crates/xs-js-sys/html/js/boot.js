function addScript(scriptUrl) {
  return new Promise((resolve, reject) => {
    const scriptNode = document.createElement("script");
    scriptNode.type = "text/javascript";
    scriptNode.src = scriptUrl;
    scriptNode.addEventListener("load", resolve);
    scriptNode.addEventListener("error", reject);
    document.head.appendChild(scriptNode);
  });
}
window.addScript = addScript;
