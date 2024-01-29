import { App, Plugin, PluginSettingTab, request, Setting } from "obsidian";
import { FileChecker } from "./file-checker";
import { Vault } from "./models/vault";

interface ConcurSettings {
	apiUrl: string;
	vaultId?: number;
	clientId?: string;
	authToken?: string;
}

const DEFAULT_SETTINGS: ConcurSettings = {
	apiUrl: "https://concur-server.shuttleapp.rs",
};

export default class ConcurPlugin extends Plugin {
	settings: ConcurSettings;
	fileChecker: FileChecker;

	async onload() {
		await this.loadSettings();

		if (!this.settings.vaultId) {
			throw new Error("Could not get vault ID");
		}

		console.log("Concur: Loaded plugin");
		console.log("Concur: Vault ID", this.settings.vaultId);
		this.fileChecker = new FileChecker(this, this.settings.vaultId);

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

		if (!data.vaultId) {
			let vault: Vault = {
				name: this.app.vault.getName(),
			};

			if (!data.apiUrl) {
				console.warn("Concur: No API URL set");
				return;
			}

			if (!data.authToken) {
				console.warn("Concur: No auth token set");
				return;
			}

			let resp: string;

			try {
				console.log("Concur: Creating vault");
				resp = await request({
					url: `${data.apiUrl}/vault`,
					method: "POST",
					headers: {
						Authorization: `Bearer ${data.authToken}`,
					},
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

			console.log("Concur: Got vault ID", vault.id);

			data.vaultId = vault.id;
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

		containerEl.createEl("button", { text: "Validate" }, (el) =>
			el.onClickEvent(async () => {
				let resp: string;
				try {
					console.log("Concur: Validating address");
					resp = await request({
						url: `${this.plugin.settings.apiUrl}/auth/client_id`,
						method: "POST",
					});
				} catch (e) {
					console.warn("Concur: Could not start session");
					console.error(e);
					return;
				}

				const { clientId } = JSON.parse(resp);
				this.plugin.settings.clientId = clientId;
				await this.plugin.saveSettings();

				el.toggleVisibility(false);

				containerEl.createEl(
					"a",
					{
						text: "Login",
						href: `${this.plugin.settings.apiUrl}/auth?client_id=${clientId}`,
					},
					(el) =>
						el.onClickEvent(async () => {
							console.log("Concur: Starting session");
							let startResp: string;
							try {
								startResp = await request({
									url: `${this.plugin.settings.apiUrl}/auth/token?client_id=${clientId}`,
									method: "GET",
								});
							} catch (e) {
								console.warn("Concur: Could not start session");
								console.error(e);
								return;
							}

							const { accessToken } = JSON.parse(startResp);
							console.log(
								"Concur: Got access token",
								accessToken,
							);
							this.plugin.settings.authToken = accessToken;
							await this.plugin.saveSettings();
						}),
				);
			}),
		);
	}
}
