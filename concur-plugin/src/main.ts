import { App, Plugin, PluginSettingTab, request, Setting } from "obsidian";
import { FileChecker } from "./file-checker";
import { Vault } from "./models/vault";

interface ConcurSettings {
	apiUrl: string;
	vault_id?: number;
}

const DEFAULT_SETTINGS: ConcurSettings = {
	apiUrl: "https://concur-server.shuttleapp.rs",
};

export default class ConcurPlugin extends Plugin {
	settings: ConcurSettings;
	fileChecker: FileChecker;

	async onload() {
		await this.loadSettings();

		if (!this.settings.vault_id) {
			throw new Error("Could not get vault ID");
		}

		this.fileChecker = new FileChecker(this, this.settings.vault_id);

		// This adds a status bar item to the bottom of the app. Does not work on mobile apps.
		const statusBarItemEl = this.addStatusBarItem();
		statusBarItemEl.setText("Status Bar Text");

		// This adds a settings tab so the user can configure various aspects of the plugin
		this.addSettingTab(new ConcurSettingTab(this.app, this));

		// When registering intervals, this function will automatically clear the interval when the plugin is disabled.
		this.registerInterval(
			window.setInterval(() => this.fileChecker.checkForChanges(), 5_000),
		);
	}

	onunload() {}

	async loadSettings() {
		await this.initSettings();
		await this.saveSettings();
	}

	async initSettings() {
		const data: ConcurSettings = Object.assign(
			{},
			DEFAULT_SETTINGS,
			await this.loadData(),
		);

		this.settings = data;

		if (!data.vault_id) {
			let vault: Vault = {
				name: this.app.vault.getName(),
			};

			if (!data.apiUrl) {
				console.warn("Concur: No API URL set");
				return;
			}

			let resp: string;

			try {
				resp = await request({
					url: `${data.apiUrl}/vault`,
					method: "POST",
					body: JSON.stringify(vault),
					contentType: "application/json",
				});
			} catch (e) {
				console.warn("Concur: Could not create vault");
				return;
			}

			vault = JSON.parse(resp);

			if (!vault.id) {
				console.warn("Concur: Could not get vault ID");
				return;
			}

			data.vault_id = vault.id;
		}
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}
}

class ConcurSettingTab extends PluginSettingTab {
	plugin: ConcurPlugin;

	constructor(app: App, plugin: ConcurPlugin) {
		super(app, plugin);
		this.plugin = plugin;
	}

	display(): void {
		const { containerEl } = this;

		containerEl.empty();

		new Setting(containerEl)
			.setName("Server URL")
			.setDesc("The URL of the server to send changes to.")
			.addText((text) =>
				text
					.setPlaceholder("Address")
					.setValue(this.plugin.settings.apiUrl)
					.onChange(async (value) => {
						this.plugin.settings.apiUrl = value;
						await this.plugin.saveSettings();
					}),
			);
	}
}
