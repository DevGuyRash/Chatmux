function workspaceRow(page, name) {
  return page
    .getByRole("button")
    .filter({ has: page.getByText(name, { exact: true }) });
}

function workspaceItem(page, name) {
  return page
    .locator(".workspace-row")
    .filter({ has: page.getByText(name, { exact: true }) });
}

module.exports = { workspaceItem, workspaceRow };
