#!/usr/bin/env python3
"""Re-render the inspected PDF pages; rendering alone is not visual verification."""
import argparse
import json
from pathlib import Path
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    import fitz  # PyMuPDF; optional reproduction prerequisite, not a runtime dependency.
    out = args.output or Path(tempfile.mkdtemp(prefix="ub-design-pages-"))
    out.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((HERE / "manifest.json").read_text())
    for item in manifest["pdfPagesActuallyViewed"]:
        with fitz.open(ROOT / item["path"]) as doc:
            page = doc[item["pdfPageOneBased"] - 1]
            page.get_pixmap(matrix=fitz.Matrix(1.55, 1.55), alpha=False).save(
                out / f"{item['id']}.png")
    print(json.dumps({"output": str(out), "pages": len(manifest["pdfPagesActuallyViewed"]),
                      "note": "Files rendered; no automatic claim of human verification."}))


if __name__ == "__main__":
    main()
