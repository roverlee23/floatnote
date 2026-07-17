import assert from "node:assert/strict";
import { shouldDeleteTable, tableAction, tableMenuActions, type TableAction } from "../src/table-actions.ts";

assert.equal(shouldDeleteTable("row", 1, 3), true);
assert.equal(shouldDeleteTable("row", 2, 3), false);
assert.equal(shouldDeleteTable("column", 3, 1), true);
assert.equal(shouldDeleteTable("column", 3, 2), false);

assert.equal(tableAction("addRow"), "addRowAfter");
assert.equal(tableAction("deleteRow"), "deleteRow");
assert.equal(tableAction("addColumn"), "addColumnAfter");
assert.equal(tableAction("deleteColumn"), "deleteColumn");
assert.equal(tableAction("deleteTable"), "deleteTable");

assert.deepEqual(tableMenuActions(), [
  "addRow",
  "deleteRow",
  "addColumn",
  "deleteColumn",
  "deleteTable",
] satisfies TableAction[]);

console.log("table action behavior: PASS");
