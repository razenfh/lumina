// src/main.ts
import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import { LazyStore } from "@tauri-apps/plugin-store";

const store = new LazyStore("settings.json");

// ---------------- Models list ----------------
const CUSTOM = "__custom__";

const MODELS: Record<string, { id: string; label: string }[]> = {
  openai: [
    { id: "gpt-4o-mini", label: "gpt-4o-mini (fast/cheap)" },
    { id: "gpt-4o", label: "gpt-4o (strong)" }
  ],
  gemini: [
    // актуальные модели из твоего ListModels
    { id: "gemini-2.5-flash", label: "gemini-2.5-flash (fast)" },
    { id: "gemini-2.5-pro", label: "gemini-2.5-pro (strong)" },
    { id: "gemini-2.5-flash-lite", label: "gemini-2.5-flash-lite (cheap)" },
    { id: "gemini-2.0-flash", label: "gemini-2.0-flash" },
    { id: "gemini-2.0-flash-lite", label: "gemini-2.0-flash-lite" }
  ],
  deepseek: [{ id: "deepseek-chat", label: "deepseek-chat" }],
  ollama: [
    { id: "llama3", label: "llama3" },
    { id: "llava", label: "llava (vision)" }
  ]
};

// ---------------- UI ----------------
const app = document.querySelector<HTMLDivElement>("#app")!;
app.innerHTML = `
  <div class="container">
    <header class="header">
      <h1>Lumina</h1>
      <button id="btnSettings">Settings</button>
    </header>

    <section class="panel">
      <div class="row">
        <button id="btnCapture">Select area</button>
        <span id="status"></span>
      </div>

      <div class="preview">
        <img id="previewImg" alt="preview" />
      </div>
    </section>

    <section class="panel">
      <div class="row wrap">
        <label>
          Provider
          <select id="provider">
            <option value="openai">OpenAI</option>
            <option value="gemini">Gemini</option>
            <option value="deepseek">DeepSeek</option>
            <option value="ollama">Ollama (local)</option>
          </select>
        </label>

        <label class="grow">
          Model
          <select id="modelSelect"></select>
        </label>

        <label class="grow" id="customModelWrap" style="display:none;">
          Custom model
          <input id="customModel" placeholder="type model name..." />
        </label>
      </div>

      <label>
        Prompt
        <textarea id="prompt" rows="4" placeholder="Explain what's in the selected area..."></textarea>
      </label>

      <div class="row">
        <button id="btnAsk">Ask AI</button>
        <span id="askStatus"></span>
      </div>

      <pre id="answer" class="answer"></pre>
    </section>

    <dialog id="settingsDialog">
      <form method="dialog" class="dialog">
        <h2>Settings</h2>
        <p class="muted">API keys are stored locally in <code>settings.json</code>.</p>

        <label>
          OpenAI API key
          <input id="openaiKey" type="password" placeholder="sk-..." />
        </label>

        <label>
          Gemini API key
          <input id="geminiKey" type="password" placeholder="..." />
        </label>

        <label>
          DeepSeek API key
          <input id="deepseekKey" type="password" placeholder="..." />
        </label>

        <div class="row right">
          <button value="cancel">Close</button>
          <button id="btnSaveSettings" value="default">Save</button>
        </div>
      </form>
    </dialog>
  </div>
`;

// ---------------- Elements ----------------
const btnCapture = document.querySelector<HTMLButtonElement>("#btnCapture")!;
const statusEl = document.querySelector<HTMLSpanElement>("#status")!;
const img = document.querySelector<HTMLImageElement>("#previewImg")!;

const providerEl = document.querySelector<HTMLSelectElement>("#provider")!;
const modelSelect = document.querySelector<HTMLSelectElement>("#modelSelect")!;
const customModelWrap = document.querySelector<HTMLLabelElement>("#customModelWrap")!;
const customModel = document.querySelector<HTMLInputElement>("#customModel")!;

const promptEl = document.querySelector<HTMLTextAreaElement>("#prompt")!;
const btnAsk = document.querySelector<HTMLButtonElement>("#btnAsk")!;
const askStatusEl = document.querySelector<HTMLSpanElement>("#askStatus")!;
const answerEl = document.querySelector<HTMLPreElement>("#answer")!;

const btnSettings = document.querySelector<HTMLButtonElement>("#btnSettings")!;
const settingsDialog = document.querySelector<HTMLDialogElement>("#settingsDialog")!;
const btnSaveSettings = document.querySelector<HTMLButtonElement>("#btnSaveSettings")!;
const openaiKeyEl = document.querySelector<HTMLInputElement>("#openaiKey")!;
const geminiKeyEl = document.querySelector<HTMLInputElement>("#geminiKey")!;
const deepseekKeyEl = document.querySelector<HTMLInputElement>("#deepseekKey")!;

// ---------------- Helpers ----------------
function getImageBase64FromImgSrc(): string {
  const src = img.src || "";
  const parts = src.split(",");
  return parts.length >= 2 ? parts[1] : "";
}

function getSelectedModel(): string {
  const v = modelSelect.value;
  if (v === CUSTOM) return customModel.value.trim();
  return v;
}

function fillModels(provider: string, prefer?: string) {
  const list = MODELS[provider] ?? [];
  modelSelect.innerHTML = "";

  for (const m of list) {
    const opt = document.createElement("option");
    opt.value = m.id;
    opt.textContent = m.label;
    modelSelect.appendChild(opt);
  }

  const customOpt = document.createElement("option");
  customOpt.value = CUSTOM;
  customOpt.textContent = "Custom…";
  modelSelect.appendChild(customOpt);

  // choose prefer
  if (prefer && list.some(x => x.id === prefer)) {
    modelSelect.value = prefer;
    customModelWrap.style.display = "none";
  } else if (prefer && prefer.trim()) {
    modelSelect.value = CUSTOM;
    customModel.value = prefer;
    customModelWrap.style.display = "block";
  } else {
    modelSelect.value = list[0]?.id ?? CUSTOM;
    customModelWrap.style.display = modelSelect.value === CUSTOM ? "block" : "none";
  }
}

async function loadUiFromStore() {
  const provider = (await store.get<string>("provider")) ?? "openai";
  let model = (await store.get<string>("model")) ?? "gpt-4o-mini";
  const prompt = (await store.get<string>("prompt")) ?? "Explain what is shown in the selected region.";

  // ---- migrate old Gemini ids (чтобы не ловить 404 из settings.json) ----
  if (model === "gemini-1.5-flash") model = "gemini-2.5-flash";
  if (model === "gemini-1.5-pro") model = "gemini-2.5-pro";

  providerEl.value = provider;
  fillModels(provider, model);
  promptEl.value = prompt;

  // если реально произошла миграция — сохраним обновлённую модель
  await store.set("provider", providerEl.value);
  await store.set("model", getSelectedModel());
  await store.set("prompt", promptEl.value);
  await store.save();
}

async function saveUiToStore() {
  await store.set("provider", providerEl.value);
  await store.set("model", getSelectedModel());
  await store.set("prompt", promptEl.value);
  await store.save();
}

function setBusyCapture(isBusy: boolean) {
  btnCapture.disabled = isBusy;
  if (isBusy) {
    statusEl.textContent = "Selecting area…";
  }
}

function setBusyAsk(isBusy: boolean) {
  btnAsk.disabled = isBusy;
  askStatusEl.textContent = isBusy ? "Sending…" : "";
}

// ---------------- Events ----------------
btnCapture.addEventListener("click", async () => {
  setBusyCapture(true);
  img.classList.remove("show");
  img.removeAttribute("src");
  answerEl.textContent = "";
  answerEl.classList.remove("show");

  try {
    const b64 = await invoke<string>("capture_region");
    if (!b64 || b64.length < 100) throw new Error("Empty image data");
    img.src = `data:image/png;base64,${b64}`;
    img.classList.add("show");
    statusEl.textContent = "Done ✅";
  } catch (e) {
    console.error(e);
    statusEl.textContent = `Error: ${String(e)}`;
  } finally {
    setBusyCapture(false);
  }
});

btnAsk.addEventListener("click", async () => {
  setBusyAsk(true);
  answerEl.textContent = "";
  answerEl.classList.remove("show");

  const image_base64 = getImageBase64FromImgSrc();
  if (!image_base64) {
    askStatusEl.textContent = "Select an area first.";
    setBusyAsk(false);
    return;
  }

  const model = getSelectedModel();
  if (!model) {
    askStatusEl.textContent = "Select a model.";
    setBusyAsk(false);
    return;
  }

  try {
    await saveUiToStore();

    const res = await invoke<string>("ask_ai", {
      req: {
        provider: providerEl.value,
        model,
        prompt: promptEl.value,
        image_base64
      }
    });

    answerEl.textContent = res;
    answerEl.classList.add("show");
    askStatusEl.textContent = "Done ✅";
  } catch (e) {
    console.error(e);
    askStatusEl.textContent = "Error";
    answerEl.textContent = String(e);
    answerEl.classList.add("show");
  } finally {
    setBusyAsk(false);
  }
});

providerEl.addEventListener("change", async () => {
  fillModels(providerEl.value);
  await saveUiToStore();
});

modelSelect.addEventListener("change", async () => {
  customModelWrap.style.display = modelSelect.value === CUSTOM ? "block" : "none";
  await saveUiToStore();
});

customModel.addEventListener("change", saveUiToStore);

// Settings
btnSettings.addEventListener("click", async () => {
  openaiKeyEl.value = (await store.get<string>("openai_api_key")) ?? "";
  geminiKeyEl.value = (await store.get<string>("gemini_api_key")) ?? "";
  deepseekKeyEl.value = (await store.get<string>("deepseek_api_key")) ?? "";
  settingsDialog.showModal();
});

btnSaveSettings.addEventListener("click", async (ev) => {
  ev.preventDefault();

  await store.set("openai_api_key", openaiKeyEl.value.trim());
  await store.set("gemini_api_key", geminiKeyEl.value.trim());
  await store.set("deepseek_api_key", deepseekKeyEl.value.trim());
  await store.save();

  settingsDialog.close();
});

// init
loadUiFromStore().catch(console.error);
