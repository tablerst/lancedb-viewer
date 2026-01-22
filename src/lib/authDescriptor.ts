import type { AuthDescriptor } from "../ipc/v1"
import { getCredential } from "./credentialVault"

export async function resolveAuthDescriptor(
	auth?: AuthDescriptor
): Promise<AuthDescriptor> {
	if (!auth || auth.type === "none") {
		return { type: "none" }
	}
	if (auth.type === "inline") {
		return auth
	}
	const credential = await getCredential(auth.reference)
	if (!credential) {
		throw new Error(`未找到凭证引用：${auth.reference}`)
	}
	return {
		type: "inline",
		provider: auth.provider,
		params: credential.params,
	}
}
