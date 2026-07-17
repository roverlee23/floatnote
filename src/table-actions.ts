export type TableAxis = "row" | "column";
export type TableAction = "addRow" | "deleteRow" | "addColumn" | "deleteColumn" | "deleteTable";

export function tableMenuActions(): TableAction[] {
  return ["addRow", "deleteRow", "addColumn", "deleteColumn", "deleteTable"];
}

export function shouldDeleteTable(axis: TableAxis, rows: number, columns: number): boolean {
  return axis === "row" ? rows <= 1 : columns <= 1;
}

export function tableAction(action: TableAction):
  | "addRowAfter"
  | "deleteRow"
  | "addColumnAfter"
  | "deleteColumn"
  | "deleteTable" {
  switch (action) {
    case "addRow":
      return "addRowAfter";
    case "deleteRow":
      return "deleteRow";
    case "addColumn":
      return "addColumnAfter";
    case "deleteColumn":
      return "deleteColumn";
    case "deleteTable":
      return "deleteTable";
  }
}
