import { print } from "./env";

// AssemblyScript uses WTF-16 by default, but our code expects a null-terminated UTF-8 string.
function utf8_string(str: String): ArrayBuffer {
  return String.UTF8.encode(str, true);
}

export function main(): void {
  print(utf8_string("Hello from WASM"));
  print(utf8_string("Hello World"));
}
