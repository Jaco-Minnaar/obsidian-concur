import { FileSystemAdapter, normalizePath, request } from "obsidian";
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

	checkForChanges(): Promise<void> {
		if (this.busy) {
			console.log("Already busy concuring");
			return Promise.resolve();
		}

		this.busy = true;

		return this.check().finally(() => (this.busy = false));
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
		let remoteFilesJson: string;
		try {
			remoteFilesJson = await request({
				url: `${apiUrl}/file?last_sync=${lastSync}&vault_id=${this.vaultId}`,
				method: "GET",
			});
		} catch (e) {
			return;
		}

		const remoteFiles = JSON.parse(remoteFilesJson) as {
			files: ConcurFile[];
		};
		console.log("Remote files", remoteFiles.files.length);

		const files = vault.getMarkdownFiles();

		for (let i = 0; i < remoteFiles.files.length; i++) {
			const file = remoteFiles.files[i];
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

		const filesToUpdate = files.filter(
			(file) => timestamps[file.path]?.valueOf() != file.stat.mtime,
		);

		console.log(
			"Updating files",
			filesToUpdate.map((f) => f.path),
		);
		const data: ConcurFile[] = await Promise.all(
			filesToUpdate.map(async (file) => {
				const content = await vault.cachedRead(file);
				return {
					vaultId: this.vaultId,
					path: file.path,
					content: content,
				};
			}),
		);

		if (filesToUpdate.length > 0) {
			for (let i = 0; i < filesToUpdate.length; i++) {
				const updatedFile = filesToUpdate[i];
				timestamps[updatedFile.path] = updatedFile.stat.mtime;
			}
			try {
				await request({
					url: `${apiUrl}/file`,
					method: "POST",
					body: JSON.stringify(data),
					contentType: "application/json",
				});
			} catch (e) {
				return;
			}
			timestamps.lastSync = Date.now();
			await adapter.write(TIMESTAMP_FILE, JSON.stringify(timestamps));
		}
	}
}
