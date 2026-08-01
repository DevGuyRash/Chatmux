async function providerPageForCanary(support, chatmux) {
  let page = support.findPage(chatmux.context);
  const providerEnv = `CHATMUX_E2E_OPEN_${support.id.toUpperCase()}`;
  const shouldOpen =
    process.env.CHATMUX_E2E_OPEN_PROVIDERS === "1" ||
    process.env[providerEnv] === "1" ||
    (support.id === "chatgpt" && process.env.CHATMUX_E2E_OPEN_CHATGPT === "1");

  if (!page && shouldOpen) {
    page = await support.openPage(chatmux.context);
  }

  if (!page) {
    throw new Error(
      `No ${support.displayName} tab is available in the qualification browser. ` +
        `Open a signed-in tab or set CHATMUX_E2E_OPEN_PROVIDERS=1 / ${providerEnv}=1.`
    );
  }

  await page.bringToFront();
  await page.waitForLoadState("domcontentloaded");
  return page;
}

module.exports = { providerPageForCanary };
