import { request } from "obsidian";
import { ConcurFile } from "src/models/file";

type FileResponse = {
	files: ConcurFile[];
};

type FileUpdateRequest = {
	path: string;
	file: ConcurFile;
};

export async function getUnsyncedFiles(
	lastSync: number,
	vaultId: number,
	apiUrl: string,
): Promise<ConcurFile[]> {
	const resp = await request({
		url: `${apiUrl}/file?last_sync=${lastSync}&vault_id=${vaultId}`,
		method: "GET",
	});

	return (JSON.parse(resp) as FileResponse).files;
}

export async function createFile(
	file: ConcurFile,
	apiUrl: string,
): Promise<void> {
	await request({
		url: `${apiUrl}/file`,
		method: "POST",
		body: JSON.stringify(file),
		contentType: "application/json",
	});
}

export async function updateFile(
	pathName: string,
	file: ConcurFile,
	apiUrl: string,
): Promise<void> {
	const updateRequest: FileUpdateRequest = {
		path: pathName,
		file,
	};

	await request({
		url: `${apiUrl}/file`,
		method: "PUT",
		body: JSON.stringify(updateRequest),
		contentType: "application/json",
	});
}
