const { invoke } = window.__TAURI__.core;

async function loadConfig() {
  try {
    const config = await invoke("get_config");
    document.getElementById("api-key").value = config.api_key || "";

    const status = document.getElementById("api-status");
    if (config.api_key) {
      status.textContent = "已配置";
      status.className = "status ok";
    } else {
      status.textContent = "未配置";
      status.className = "status error";
    }

    const modelSelect = document.getElementById("model");
    if (config.model) {
      modelSelect.value = config.model;
    }

    document.getElementById("prompt").value = config.prompt || "";

    const devices = await invoke("list_audio_devices");
    const select = document.getElementById("mic-device");
    devices.forEach((name) => {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      if (name === config.microphone_device) {
        option.selected = true;
      }
      select.appendChild(option);
    });
  } catch (e) {
    console.error("Failed to load config:", e);
  }
}

document.getElementById("save-btn").addEventListener("click", async () => {
  const apiKey = document.getElementById("api-key").value.trim();
  const micDevice = document.getElementById("mic-device").value || null;
  const model = document.getElementById("model").value;
  const prompt = document.getElementById("prompt").value.trim();

  try {
    await invoke("save_config_cmd", {
      newConfig: {
        api_key: apiKey,
        hotkey: "CommandLeft+ShiftLeft+Space",
        microphone_device: micDevice,
        model: model,
        prompt: prompt,
      },
    });

    const saveStatus = document.getElementById("save-status");
    saveStatus.textContent = "已保存";
    setTimeout(() => { saveStatus.textContent = ""; }, 2000);

    const status = document.getElementById("api-status");
    if (apiKey) {
      status.textContent = "已配置";
      status.className = "status ok";
    } else {
      status.textContent = "未配置";
      status.className = "status error";
    }
  } catch (e) {
    console.error("Failed to save config:", e);
    const saveStatus = document.getElementById("save-status");
    saveStatus.textContent = "保存失败";
    saveStatus.style.color = "#f44336";
  }
});

loadConfig();
