#!/usr/bin/env python3
"""Re:Trace Python sidecar — handles LIVE and StarVector inference via TCP socket."""
import importlib
import json
import socket
import threading

PORT = 29371


def handle_client(conn: socket.socket) -> None:
    try:
        f = conn.makefile("r")
        line = f.readline()
        if not line:
            return
        req = json.loads(line)
        backend = req.get("backend", "")
        input_path = req.get("input", "")
        output_path = req.get("output", "")

        if backend == "live":
            mod = importlib.import_module("backends.live")
        elif backend in ("starvector-1b", "starvector-8b"):
            mod = importlib.import_module("backends.starvector")
        else:
            raise ValueError(f"Unknown backend: {backend!r}")

        mod.trace(input_path, output_path, backend)
        resp = json.dumps({"ok": True})
    except Exception as exc:
        resp = json.dumps({"ok": False, "error": str(exc)})
    finally:
        try:
            conn.sendall((resp + "\n").encode())
        except Exception:
            pass
        conn.close()


def main() -> None:
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", PORT))
    server.listen(5)
    print(f"Re:Trace sidecar listening on port {PORT}", flush=True)

    while True:
        conn, _ = server.accept()
        threading.Thread(target=handle_client, args=(conn,), daemon=True).start()


if __name__ == "__main__":
    main()
