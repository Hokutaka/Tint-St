import {
  EditorView,
} from "@codemirror/view";

import {
  minimalSetup,
} from "codemirror";

import {
  open,
  save,
} from "@tauri-apps/plugin-dialog";

import {
  getCurrentWindow,
} from "@tauri-apps/api/window";

import {
  readTextFile,
  writeTextFile,
} from "@tauri-apps/plugin-fs";

import { invoke } from "@tauri-apps/api/core";

import {
  ceruneHighlighting,
  ceruneLanguage,
} from "./cerune-language";

type Target =
  | "x86_64-pc-windows-msvc"
  | "x86_64-unknown-linux-gnu";

type OutputKind =
  | "sources"
  | "ir"
  | "c"
  | "cAsm"
  | "llvm"
  | "llvmAsm"
  | "wat"
  | "qbe"
  | "qbeAsm"
  | "directAsm"
  | "object"
  | "bytecode"
  | "vmOutput";

interface EmitResult {
  sources: string;
  ir: string;
  c: string;
  cAsm: string;

  llvm: string;
  llvmAsm: string;

  wat: string;

  qbe: string;
  qbeAsm: string;

  directAsm: string;
  object: string;

  bytecode: string;
  vmOutput: string;
}

const outputs: EmitResult = {
  sources: "",
  ir: "",
  c: "",
  cAsm: "",

  llvm: "",
  llvmAsm: "",

  wat: "",

  qbe: "",
  qbeAsm: "",

  directAsm: "",
  object: "",

  bytecode: "",
  vmOutput: "",
};

const initialSource = `integer: i64 = 1 + 2;
single: f32 = 0.1 + 0.2;
double: f64 = 0.1 + 0.2;
inferred: infer = single + single;

print(integer);
print(single);
print(double);
print(inferred);`;

let activeOutput: OutputKind = "ir";
let currentPath: string | null = null;

let sourceView:
  | EditorView
  | null = null;

function selectedTarget(): Target {
  const element =
    document.querySelector<HTMLSelectElement>(
      "#target-select",
    );

  if (!element) {
    throw new Error(
      "target selector not found",
    );
  }

  return element.value as Target;
}

function sourceText(): string {
  if (!sourceView) {
    throw new Error(
      "source editor not initialized",
    );
  }

  return sourceView.state.doc.toString();
}

function outputElement(): HTMLElement {
  const element =
    document.querySelector<HTMLElement>(
      "#output",
    );

  if (!element) {
    throw new Error(
      "output view not found",
    );
  }

  return element;
}

function setStatus(
  text: string,
  state:
    | "ready"
    | "working"
    | "error" = "ready",
) {
  const statusText =
    document.querySelector<HTMLElement>(
      "#status-text",
    );

  const statusDot =
    document.querySelector<HTMLElement>(
      "#status-dot",
    );

  if (statusText) {
    statusText.textContent = text;
  }

  if (statusDot) {
    statusDot.dataset.state = state;
  }
}

function showOutput(
  kind: OutputKind,
) {
  activeOutput = kind;

  document
    .querySelectorAll<HTMLButtonElement>(
      ".tab",
    )
    .forEach((button) => {
      button.classList.toggle(
        "active",
        button.dataset.output === kind,
      );
    });

  outputElement().textContent =
    outputs[kind] ||
    "Press Emit to generate code.";
}

function annotateOrigins(): boolean {
  const element =
    document.querySelector<HTMLInputElement>(
      "#origins-toggle",
    );

  if (!element) {
    throw new Error(
      "origins toggle not found",
    );
  }

  return element.checked;
}

async function openFile() {
  const path = await open({
    multiple: false,
    filters: [
      {
        name: "Cerune",
        extensions: ["ceru"],
      },
    ],
  });

  if (!path || Array.isArray(path)) {
    return;
  }

  const source = await readTextFile(path);

  currentPath = path;

  sourceView?.dispatch({
    changes: {
      from: 0,
      to: sourceView.state.doc.length,
      insert: source,
    },
  });

  await updateFileName(path);
  setStatus("Opened");
}

async function saveFile() {
  let path = currentPath;

  if (!path) {
    path = await save({
      filters: [
        {
          name: "Cerune",
          extensions: ["ceru"],
        },
      ],
    });
  }

  if (!path) {
    return;
  }

  await writeTextFile(
    path,
    sourceText(),
  );

  currentPath = path;

  await updateFileName(path);
  setStatus("Saved");
}

async function emit() {
  const button =
    document.querySelector<HTMLButtonElement>(
      "#emit-button",
    );

  if (button) {
    button.disabled = true;
  }

  setStatus(
    "Generating…",
    "working",
  );

  try {
    const result =
      await invoke<EmitResult>(
        "emit_all",
        {
          source: sourceText(),
          target: selectedTarget(),
          annotateOrigins: annotateOrigins(),
          sourcePath: currentPath,
        },
      );

    Object.assign(outputs, result);

    showOutput(activeOutput);

    setStatus("Ready");
  } catch (error) {
    outputElement().textContent =
      String(error);

    setStatus(
      "Cerune error",
      "error",
    );
  } finally {
    if (button) {
      button.disabled = false;
    }
  }
}

async function saveAsFile() {
  const path = await save({
    defaultPath:
      currentPath ?? "Untitled.ceru",

    filters: [
      {
        name: "Cerune",
        extensions: ["ceru"],
      },
    ],
  });

  if (!path) {
    return;
  }

  await writeTextFile(
    path,
    sourceText(),
  );

  currentPath = path;

  await updateFileName(path);

  setStatus("Saved As");
}

function fileNameFromPath(
  path: string,
): string {
  return (
    path.split(/[\\/]/).pop()
    ?? path
  );
}

async function updateFileName(
  path: string | null,
) {
  const element =
    document.querySelector<HTMLButtonElement>(
      "#file-name",
    );

  if (!element) {
    return;
  }

  if (!path) {
    element.textContent =
      "Untitled.ceru";

    element.title =
      "Click to rename";

    await getCurrentWindow()
      .setTitle("Tint* Workspace");

    return;
  }

  const name =
    fileNameFromPath(path);

  element.textContent = name;
  element.title = path;

  await getCurrentWindow()
    .setTitle(`Tint* — ${name}`);
}

const editorTheme =
  EditorView.theme({
    "&": {
      height: "100%",
      backgroundColor: "#ffffff",
      color: "#253746",
    },

    "&.cm-focused": {
      outline: "none",
    },

    ".cm-scroller": {
      fontFamily:
        '"Cascadia Code", "SFMono-Regular", Consolas, monospace',
      fontSize: "13px",
      lineHeight: "1.7",
      overflow: "auto",
    },

    ".cm-content": {
      padding: "18px",
      caretColor: "#3983ad",
    },

    ".cm-line": {
      padding: "0",
    },

    ".cm-cursor": {
      borderLeftColor: "#3983ad",
    },

    ".cm-selectionBackground": {
      backgroundColor:
        "#dff3ff !important",
    },
  });

async function renameCurrentFile() {
  if (!currentPath) {
    return;
  }

  const oldName =
    fileNameFromPath(currentPath);

  const input =
    window.prompt(
      "Rename Cerune file",
      oldName,
    );

  if (input === null) {
    return;
  }

  let newName =
    input.trim();

  if (!newName) {
    return;
  }

  if (!newName.endsWith(".ceru")) {
    newName += ".ceru";
  }

  if (newName === oldName) {
    return;
  }

  try {
    const newPath =
      await invoke<string>(
        "rename_source",
        {
          path: currentPath,
          newName,
        },
      );

    currentPath = newPath;

    await updateFileName(newPath);

    setStatus("Renamed");
  } catch (error) {
    setStatus(
      "Rename error",
      "error",
    );

    outputElement().textContent =
      String(error);
  }
}

window.addEventListener(
  "DOMContentLoaded",
  () => {
    const editorParent =
      document.querySelector<HTMLElement>(
        "#source-editor",
      );

    if (!editorParent) {
      throw new Error(
        "source editor container not found",
      );
    }

    sourceView = new EditorView({
      doc: initialSource,
      parent: editorParent,

      extensions: [
        minimalSetup,
        ceruneLanguage,
        ceruneHighlighting,
        editorTheme,
      ]
    });

    window.addEventListener(
      "keydown",
      (event) => {
        const modifier =
          event.ctrlKey ||
          event.metaKey;

        if (!modifier) {
          return;
        }

        if (event.key === "Enter") {
          event.preventDefault();
          event.stopPropagation();

          void emit();
          return;
        }

        if (
          event.key.toLowerCase() === "o"
        ) {
          event.preventDefault();
          event.stopPropagation();

          void openFile();
          return;
        }

        if (
          event.key.toLowerCase() === "s"
        ) {
          event.preventDefault();
          event.stopPropagation();

          if (event.shiftKey) {
            void saveAsFile();
          } else {
            void saveFile();
          }
        }
      },
      true,
    );

    document
      .querySelector("#open-button")
      ?.addEventListener(
        "click",
        () => {
          void openFile();
        },
      );

    document
      .querySelector("#save-button")
      ?.addEventListener(
        "click",
        () => {
          void saveFile();
        },
      );

    document
      .querySelector("#emit-button")
      ?.addEventListener(
        "click",
        () => {
          void emit();
        },
      );

    document
      .querySelectorAll<HTMLButtonElement>(
        ".tab",
      )
      .forEach((button) => {
        button.addEventListener(
          "click",
          () => {
            showOutput(
              button.dataset
                .output as OutputKind,
            );
          },
        );
      });

    document
      .querySelector("#file-name")
      ?.addEventListener(
        "click",
        () => {
          void renameCurrentFile();
        },
      );
  },
);
