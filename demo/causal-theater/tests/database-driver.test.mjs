import test from "node:test";
import assert from "node:assert/strict";
import { DatabaseDriver, checkedDispatch } from "../database-driver.js";

const authorized = () => ({ status: "AUTHORIZED", lease_consumed: true, actions: [199],
  dispatch_digest: "18446744073709551615", commitment_digest: "12345678901234567" });
test("the database driver requires an exact, consumed P9 dispatch", () => {
  assert.equal(checkedDispatch(authorized()).actions[0], 199);
  for (const change of [{ status: "ABSTAIN" }, { lease_consumed: false }, { actions: [] },
    { actions: [99] }, { actions: [199,199] }, { dispatch_digest: "" }]) {
    assert.throws(() => checkedDispatch({ ...authorized(), ...change }), /No exact P9/);
  }
});
test("unavailable storage produces no success and ambiguous driver attempts are not retried", async () => {
  const driver = new DatabaseDriver("test-only-run");
  await assert.rejects(driver.execute(authorized()), /IndexedDB unavailable/);
  assert.equal(driver.writes, 0);
  await assert.rejects(driver.execute(authorized()), /retry is withheld/);
});
