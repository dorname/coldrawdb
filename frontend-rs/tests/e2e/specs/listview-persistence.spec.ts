import { expect, test } from "@playwright/test";
import {
  API_BASE,
  debugState,
  inviteAndAccept,
  newRoomPage,
  registerAndLogin,
  waitCollabConnected,
} from "../helpers/collab";

async function createSeededRoom(
  request: Parameters<typeof registerAndLogin>[0],
  owner: Awaited<ReturnType<typeof registerAndLogin>>,
) {
  const suffix = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  const tableId = `table-list-${suffix}`;
  const fieldId = `field-list-${suffix}`;
  const created = await request.post(`${API_BASE}/api/v1/diagrams`, {
    data: { name: `list-persistence-${suffix}`, database: "postgresql" },
  });
  expect(created.ok()).toBeTruthy();
  const diagramId = (await created.json()).data.id as string;
  const loaded = await request.get(`${API_BASE}/api/v1/diagrams/${diagramId}`);
  expect(loaded.ok()).toBeTruthy();
  const diagram = (await loaded.json()).data;
  diagram.tables = [
    {
      id: tableId,
      name: "users",
      x: 80,
      y: 80,
      color: "#3b82f6",
      comment: "",
      fields: [
        {
          id: fieldId,
          name: "status",
          type_: "INT",
          default: "",
          check: "",
          primary: false,
          unique: false,
          not_null: false,
          increment: false,
          comment: "",
          tag: "",
        },
      ],
      indices: [],
    },
  ];
  const saved = await request.put(`${API_BASE}/api/v1/diagrams/${diagramId}`, {
    data: { expected_revision: diagram.revision, diagram },
  });
  expect(saved.ok()).toBeTruthy();
  const room = await request.post(`${API_BASE}/api/v1/rooms`, {
    headers: { Authorization: `Bearer ${owner.accessToken}` },
    data: { name: `list-persistence-${suffix}`, diagramId },
  });
  expect(room.status()).toBe(201);
  return { diagramId, tableId, fieldId, roomId: (await room.json()).id as string };
}

test("ST-SP-LIST-02: 全屏列表字段编辑经真实 WS 持久化并在重载后恢复", async ({
  browser,
  request,
}) => {
  test.setTimeout(60_000);
  const owner = await registerAndLogin(request, "list-persistence");
  const { diagramId, tableId, fieldId, roomId } = await createSeededRoom(request, owner);
  const roomPage = await newRoomPage(browser, owner, roomId);
  const { page } = roomPage;
  try {
    await waitCollabConnected(page);
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();

    const name = page.locator(`[data-testid="list-field-name-${fieldId}"]`);
    await name.fill("account_status");
    await name.evaluate((element: HTMLInputElement) => element.blur());
    const comment = page.locator(`[data-testid="list-field-comment-${fieldId}"]`);
    await comment.fill("账户状态");
    await comment.evaluate((element: HTMLInputElement) => element.blur());
    await page.locator(`[data-testid="list-field-pk-${fieldId}"]`).check();
    await page.locator(`[data-testid="list-field-nn-${fieldId}"]`).check();
    await page.locator(`[data-testid="list-field-uq-${fieldId}"]`).check();
    await page.locator(`[data-testid="list-field-auto-${fieldId}"]`).check();
    await page.locator(`[data-testid="list-field-type-${fieldId}"]`).selectOption("VARCHAR");
    const length = page.locator(`[data-testid="list-field-len-${fieldId}"]`);
    await length.fill("32");
    await length.evaluate((element: HTMLInputElement) => element.blur());

    const readServerField = async () => {
      const response = await request.get(`${API_BASE}/api/v1/diagrams/${diagramId}`, {
        headers: { Authorization: `Bearer ${owner.accessToken}` },
      });
      if (!response.ok()) return null;
      const body = await response.json();
      return body.data.tables
        .find((table: { id: string }) => table.id === tableId)
        ?.fields.find((field: { id: string }) => field.id === fieldId) ?? null;
    };
    await expect
      .poll(readServerField, { timeout: 10_000 })
      .toMatchObject({
        id: fieldId,
        name: "account_status",
        comment: "账户状态",
        primary: true,
        not_null: true,
        unique: true,
        increment: true,
        type_: "VARCHAR(32)",
      });

    await page.screenshot({ path: "../../../logs/issue-58-listview-persisted.png" });
    await page.goto("/");
    await page.locator('[data-testid="rooms-list-page"]').waitFor();
    await page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await page.locator('[data-testid="room-editor-page"]').waitFor();
    await waitCollabConnected(page);
    await page.locator('[data-testid="btn-list-view"]').click();
    await expect(page.locator(`[data-testid="list-field-name-${fieldId}"]`)).toHaveValue(
      "account_status",
    );
    await expect(page.locator(`[data-testid="list-field-comment-${fieldId}"]`)).toHaveValue(
      "账户状态",
    );
    await expect(page.locator(`[data-testid="list-field-pk-${fieldId}"]`)).toBeChecked();
    await expect(page.locator(`[data-testid="list-field-nn-${fieldId}"]`)).toBeChecked();
    await expect(page.locator(`[data-testid="list-field-uq-${fieldId}"]`)).toBeChecked();
    await expect(page.locator(`[data-testid="list-field-auto-${fieldId}"]`)).toBeChecked();
    await expect(page.locator(`[data-testid="list-field-len-${fieldId}"]`)).toHaveValue("32");
    expect((await debugState(page)).dirty).toBe(false);

    const viewer = await registerAndLogin(request, "list-persistence-viewer");
    await inviteAndAccept(request, owner, viewer, roomId, "viewer");
    const viewerRoom = await newRoomPage(browser, viewer, roomId);
    try {
      await waitCollabConnected(viewerRoom.page);
      await viewerRoom.page.locator('[data-testid="btn-list-view"]').click();
      const viewerName = viewerRoom.page.locator(`[data-testid="list-field-name-${fieldId}"]`);
      const viewerNotNull = viewerRoom.page.locator(`[data-testid="list-field-nn-${fieldId}"]`);
      await expect(viewerName).toBeDisabled();
      await expect(viewerNotNull).toBeDisabled();
      await viewerNotNull.evaluate((element: HTMLInputElement) => {
        element.checked = false;
        element.dispatchEvent(new Event("change", { bubbles: true }));
      });
      expect((await readServerField())?.not_null).toBe(true);
    } finally {
      await viewerRoom.context.close();
    }
  } finally {
    await roomPage.context.close();
  }
});
