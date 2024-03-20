import {
	FileSystemAdapter,
	normalizePath,
	TAbstractFile,
	TFile,
} from "obsidian";
import { getUnsyncedFiles, createFile, updateFile } from "./api/file";
import ConcurPlugin from "./main";
import { ConcurFile } from "./models/file";

type Timestamps = { [path: string]: number };

const CONCUR_DIR = normalizePath(".concur");
const TIMESTAMP_FILE = `${CONCUR_DIR}/concur_timestamps.json`;
export class FileChecker {
	private busy = false;

	constructor(
		private readonly plugin: ConcurPlugin,
		private readonly vaultId: number,
	) {}

	async checkForChanges(): Promise<void> {
		if (this.busy) {
			console.log("Already busy concuring");
			return Promise.resolve();
		}

		this.busy = true;

		return this.check().finally(() => (this.busy = false));
	}

	async saveFile(file: TAbstractFile): Promise<void> {
		const concurFile: ConcurFile = {
			vaultId: this.vaultId,
			path: file.path,
			content: "",
		};

		await createFile(concurFile, this.plugin.settings.apiUrl);
	}

	async updateFile(
		file: TFile,
		data: string,
		filePath?: string,
	): Promise<void> {
		filePath ??= file.path;

		const concurFile: ConcurFile = {
			vaultId: this.vaultId,
			path: file.path,
			content: data,
		};

		await updateFile(filePath, concurFile, this.plugin.settings.apiUrl);
	}

	private async check(): Promise<void> {
		const apiUrl = this.plugin.settings.apiUrl;
		const vault = this.plugin.app.vault;
		const adapter = vault.adapter as FileSystemAdapter;

		if (!(await vault.adapter.exists(CONCUR_DIR))) {
			await adapter.mkdir(CONCUR_DIR);
		}

		let timestamps: Timestamps;
		if (!(await adapter.exists(TIMESTAMP_FILE))) {
			timestamps = {};
		} else {
			timestamps = JSON.parse(await adapter.read(TIMESTAMP_FILE));
		}

		const lastSync = Math.floor(timestamps.lastSync / 1000) || 0;
		let remoteFiles: ConcurFile[];
		try {
			remoteFiles = await getUnsyncedFiles(
				lastSync,
				this.vaultId,
				apiUrl,
			);
		} catch (e) {
			return;
		}

		console.log("Remote files", remoteFiles.length);

		const files = vault.getMarkdownFiles();

		for (let i = 0; i < remoteFiles.length; i++) {
			const file = remoteFiles[i];
			const existing = files.find((f) => f.path === file.path);

			if (existing) {
				await vault.modify(existing, file.content);
			} else {
				const normalizedPath = normalizePath(
					file.path.split("/").slice(0, -1).join("/"),
				);
				if (!(await vault.adapter.exists(normalizedPath))) {
					await vault.adapter.mkdir(normalizedPath);
				}
				await vault.create(file.path, file.content);
			}
		}
	}
}
