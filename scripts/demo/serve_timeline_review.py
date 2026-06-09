#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


class RangeRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self) -> None:
        self.send_header("Accept-Ranges", "bytes")
        super().end_headers()

    def do_HEAD(self) -> None:
        self._send_file(send_body=False)

    def do_GET(self) -> None:
        self._send_file(send_body=True)

    def _send_file(self, *, send_body: bool) -> None:
        path = Path(self.translate_path(self.path))
        if path.is_dir() or not path.exists():
            super().do_GET() if send_body else super().do_HEAD()
            return

        size = path.stat().st_size
        range_header = self.headers.get("Range")
        if not range_header:
            super().do_GET() if send_body else super().do_HEAD()
            return

        parsed = self._parse_range(range_header, size)
        if parsed is None:
            self.send_error(HTTPStatus.REQUESTED_RANGE_NOT_SATISFIABLE)
            return

        start, end = parsed
        length = end - start + 1
        self.send_response(HTTPStatus.PARTIAL_CONTENT)
        self.send_header("Content-type", self.guess_type(str(path)))
        self.send_header("Content-Length", str(length))
        self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
        self.send_header("Last-Modified", self.date_time_string(path.stat().st_mtime))
        self.end_headers()
        if not send_body:
            return

        with path.open("rb") as handle:
            handle.seek(start)
            remaining = length
            while remaining > 0:
                chunk = handle.read(min(1024 * 1024, remaining))
                if not chunk:
                    break
                self.wfile.write(chunk)
                remaining -= len(chunk)

    @staticmethod
    def _parse_range(header: str, size: int) -> tuple[int, int] | None:
        match = re.fullmatch(r"bytes=(\d*)-(\d*)", header.strip())
        if not match or size <= 0:
            return None
        start_raw, end_raw = match.groups()
        if start_raw == "" and end_raw == "":
            return None
        if start_raw == "":
            suffix = int(end_raw)
            if suffix <= 0:
                return None
            start = max(0, size - suffix)
            end = size - 1
        else:
            start = int(start_raw)
            end = int(end_raw) if end_raw else size - 1
        if start >= size or end < start:
            return None
        return start, min(end, size - 1)


def main() -> int:
    parser = argparse.ArgumentParser(description="Serve the tutorial timeline review with HTTP byte-range support.")
    parser.add_argument("--directory", default="demo-artifacts/review-human")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18085)
    args = parser.parse_args()

    os.chdir(Path(args.directory).resolve())
    server = ThreadingHTTPServer((args.host, args.port), RangeRequestHandler)
    print(f"serving {Path.cwd()} on http://{args.host}:{args.port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        return 130
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
