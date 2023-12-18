import { FileSystemAdapter, normalizePath, Plugin, request } from "obsidian";
import { ConcurFile } from "./models/file";
import { createHash } from "node:crypto";

type Timestamps = { [path: string]: number };

const CONCUR_DIR = normalizePath(".concur");
const TIMESTAMP_FILE = `${CONCUR_DIR}/concur_timestamps.json`;
export class FileChecker {
	constructor(
		private readonly plugin: Plugin,
		private readonly vaultId: number,
	) {}

	async checkForChanges() {
		const vault = this.plugin.app.vault;
		const adapter = vault.adapter as FileSystemAdapter;

		if (!vault.adapter.exists(CONCUR_DIR)) {
			adapter.mkdir(CONCUR_DIR);
		}

		let timestamps: Timestamps;
		if (!adapter.exists(TIMESTAMP_FILE)) {
			timestamps = {};
		} else {
			timestamps = JSON.parse(await adapter.read(TIMESTAMP_FILE));
		}

		// const lastSync = Math.floor(timestamps.lastSync / 1000 ?? 0);
		const lastSync = 0;
		const remoteFilesJson = await request({
			url: `http://localhost:3000/file?last_sync=${lastSync}`,
			method: "GET",
		});

		const remoteFiles = JSON.parse(remoteFilesJson) as ConcurFile[];

		const files = vault.getMarkdownFiles();

		for (let i = 0; i < remoteFiles.length; i++) {
			const file = remoteFiles[i];
			const existing = files.find((f) => f.path === file.path);

			if (existing) {
				vault.modify(existing, file.content);
			} else {
				vault.create(file.path, file.content);
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
				const hash = createHash("md5");
				const content = await vault.cachedRead(file);
				return {
					vaultId: this.vaultId,
					path: file.path,
					content: content,
					hash: hash.update(content).digest("base64"),
				};
			}),
		);

		if (filesToUpdate.length > 0) {
			for (let i = 0; i < filesToUpdate.length; i++) {
				const updatedFile = filesToUpdate[i];
				timestamps[updatedFile.path] = updatedFile.stat.mtime;
			}
			await request({
				url: "http://localhost:3000/file",
				method: "POST",
				body: JSON.stringify(data),
				contentType: "application/json",
			});
			timestamps.lastSync = Date.now();
			await adapter.write(TIMESTAMP_FILE, JSON.stringify(timestamps));
		}
	}
}
