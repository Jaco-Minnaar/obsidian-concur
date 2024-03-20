import { request } from "obsidian";
import { Vault } from "src/models/vault";

export async function getVault(name: string, apiUrl: string): Promise<Vault> {
	const vault: Vault = {
		name,
	};

	console.log("Creating vault", vault, "at", apiUrl);

	const resp = await request({
		url: `${apiUrl}/vault`,
		method: "POST",
		body: JSON.stringify(vault),
		contentType: "application/json",
	});

	return JSON.parse(resp);
}
