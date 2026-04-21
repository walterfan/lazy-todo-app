/**
 * Crypto / utility helpers for the Toolbox.
 * All operations are client-side; nothing is persisted or sent over the network.
 */

export const MAX_INPUT_BYTES = 1_048_576;

export class InputTooLargeError extends Error {
  constructor(public readonly bytes: number) {
    super(`Input exceeds ${MAX_INPUT_BYTES} bytes (got ${bytes}).`);
    this.name = "InputTooLargeError";
  }
}

export function assertInputSize(text: string): void {
  const bytes = new TextEncoder().encode(text).length;
  if (bytes > MAX_INPUT_BYTES) {
    throw new InputTooLargeError(bytes);
  }
}

// ---------- Encoding helpers ----------

export function bytesToHex(bytes: Uint8Array): string {
  let out = "";
  for (let i = 0; i < bytes.length; i++) {
    out += bytes[i].toString(16).padStart(2, "0");
  }
  return out;
}

export function hexToBytes(hex: string): Uint8Array {
  const clean = hex.trim().replace(/\s+/g, "");
  if (clean.length % 2 !== 0) {
    throw new Error("Hex string must have an even length.");
  }
  if (!/^[0-9a-fA-F]*$/.test(clean)) {
    throw new Error("Hex string contains non-hex characters.");
  }
  const out = new Uint8Array(clean.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(clean.substr(i * 2, 2), 16);
  }
  return out;
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
  return btoa(binary);
}

export function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64.trim());
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
}

export function base64UrlEncode(bytes: Uint8Array): string {
  return bytesToBase64(bytes).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

// ---------- Digest helpers ----------

type NativeHash = "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512";

async function nativeDigest(algorithm: NativeHash, text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buf = await crypto.subtle.digest(algorithm, data);
  return bytesToHex(new Uint8Array(buf));
}

export type HashAlgorithm = "MD5" | NativeHash;

export async function sha(algorithm: HashAlgorithm, text: string): Promise<string> {
  assertInputSize(text);
  if (algorithm === "MD5") return md5(text);
  return nativeDigest(algorithm, text);
}

// ---------- HMAC-SHA256 (for JWT HS256) ----------

export async function hmacSha256(key: string, data: string): Promise<Uint8Array> {
  const enc = new TextEncoder();
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    enc.encode(key),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const sig = await crypto.subtle.sign("HMAC", cryptoKey, enc.encode(data));
  return new Uint8Array(sig);
}

export async function hmacSha256Base64Url(key: string, data: string): Promise<string> {
  return base64UrlEncode(await hmacSha256(key, data));
}

// ---------- AES (CBC / GCM) ----------

export type AesMode = "AES-CBC" | "AES-GCM";

function validateAesKey(keyHex: string): Uint8Array {
  const bytes = hexToBytes(keyHex);
  if (![16, 24, 32].includes(bytes.length)) {
    throw new Error("AES key must be 128, 192, or 256 bits (16/24/32 bytes of hex).");
  }
  return bytes;
}

function validateAesIv(mode: AesMode, ivHex: string): Uint8Array {
  const bytes = hexToBytes(ivHex);
  const expected = mode === "AES-CBC" ? 16 : 12;
  if (bytes.length !== expected) {
    throw new Error(`${mode} IV must be ${expected} bytes (${expected * 2} hex chars).`);
  }
  return bytes;
}

export async function aesEncrypt(
  mode: AesMode,
  keyHex: string,
  ivHex: string,
  plaintext: string
): Promise<string> {
  assertInputSize(plaintext);
  const keyBytes = validateAesKey(keyHex);
  const iv = validateAesIv(mode, ivHex);
  const cryptoKey = await crypto.subtle.importKey("raw", keyBytes, { name: mode }, false, ["encrypt"]);
  const params = mode === "AES-CBC" ? { name: "AES-CBC", iv } : { name: "AES-GCM", iv };
  const cipher = await crypto.subtle.encrypt(params, cryptoKey, new TextEncoder().encode(plaintext));
  return bytesToBase64(new Uint8Array(cipher));
}

export async function aesDecrypt(
  mode: AesMode,
  keyHex: string,
  ivHex: string,
  ciphertextBase64: string
): Promise<string> {
  const keyBytes = validateAesKey(keyHex);
  const iv = validateAesIv(mode, ivHex);
  const cryptoKey = await crypto.subtle.importKey("raw", keyBytes, { name: mode }, false, ["decrypt"]);
  const params = mode === "AES-CBC" ? { name: "AES-CBC", iv } : { name: "AES-GCM", iv };
  try {
    const plain = await crypto.subtle.decrypt(params, cryptoKey, base64ToBytes(ciphertextBase64));
    return new TextDecoder().decode(plain);
  } catch {
    throw new Error("Decryption failed — wrong key, IV, or corrupted input.");
  }
}

// ---------- Clipboard ----------

export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

// ---------- Pure-JS MD5 (RFC 1321) ----------
// Small, self-contained implementation. Only exposed via `sha("MD5", ...)`.
// MD5 is cryptographically broken — use SHA-256 for security-critical work.

function md5(text: string): string {
  const bytes = new TextEncoder().encode(text);
  return md5Bytes(bytes);
}

function md5Bytes(bytes: Uint8Array): string {
  const bitLen = bytes.length * 8;
  const blockCount = Math.floor((bytes.length + 8) / 64) + 1;
  const padded = new Uint8Array(blockCount * 64);
  padded.set(bytes);
  padded[bytes.length] = 0x80;
  // Low 32 bits of length in bits.
  const lenLow = bitLen >>> 0;
  const lenHigh = Math.floor(bitLen / 0x100000000) >>> 0;
  const dv = new DataView(padded.buffer);
  dv.setUint32(padded.length - 8, lenLow, true);
  dv.setUint32(padded.length - 4, lenHigh, true);

  let a0 = 0x67452301;
  let b0 = 0xefcdab89;
  let c0 = 0x98badcfe;
  let d0 = 0x10325476;

  const K = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
  ];
  const S = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
  ];

  const rotl = (x: number, n: number) => ((x << n) | (x >>> (32 - n))) >>> 0;

  for (let block = 0; block < blockCount; block++) {
    const M = new Array<number>(16);
    for (let i = 0; i < 16; i++) {
      M[i] = dv.getUint32(block * 64 + i * 4, true);
    }
    let A = a0, B = b0, C = c0, D = d0;

    for (let i = 0; i < 64; i++) {
      let F: number;
      let g: number;
      if (i < 16) {
        F = (B & C) | (~B & D);
        g = i;
      } else if (i < 32) {
        F = (D & B) | (~D & C);
        g = (5 * i + 1) % 16;
      } else if (i < 48) {
        F = B ^ C ^ D;
        g = (3 * i + 5) % 16;
      } else {
        F = C ^ (B | ~D);
        g = (7 * i) % 16;
      }
      F = (F + A + K[i] + M[g]) >>> 0;
      A = D;
      D = C;
      C = B;
      B = (B + rotl(F, S[i])) >>> 0;
    }

    a0 = (a0 + A) >>> 0;
    b0 = (b0 + B) >>> 0;
    c0 = (c0 + C) >>> 0;
    d0 = (d0 + D) >>> 0;
  }

  const out = new Uint8Array(16);
  const outDv = new DataView(out.buffer);
  outDv.setUint32(0, a0, true);
  outDv.setUint32(4, b0, true);
  outDv.setUint32(8, c0, true);
  outDv.setUint32(12, d0, true);
  return bytesToHex(out);
}
