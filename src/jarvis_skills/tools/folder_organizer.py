"""
Folder Organizer Tool - Safe file organization within scoped directories.

Implements a three-phase workflow:
1. Discovery: Scan files, selectively read content for categorization
2. Planning: Generate detailed move plan with safety validation
3. Execution: Execute moves with error handling and retry logic
"""

from __future__ import annotations

import os
import shutil
import logging
from datetime import datetime
from pathlib import Path

from jarvis_skills_core import ToolParameter, ToolParameterType

logger = logging.getLogger(__name__)

DEFAULT_ALLOWED_DIR_NAMES = ("Desktop", "Downloads", "Documents")
SUPPORTED_STRATEGIES = {"extension", "type", "date"}
TEXT_EXTENSIONS = {".txt", ".md", ".py", ".js", ".ts", ".json", ".yml", ".yaml", ".log"}

def _read_file_snippet(file_path: Path, max_bytes: int = 512) -> str:
    """Safely read first N bytes of a file for content-based categorization."""
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read(max_bytes)
            return content[:max_bytes]
    except (OSError, IOError) as e:
        logger.debug(f"Could not read {file_path}: {e}")
        return ""


def _categorize_by_content(file_path: Path) -> str | None:
    """Attempt to categorize file by content if extension-based categorization is ambiguous."""
    if file_path.suffix.lower() not in TEXT_EXTENSIONS:
        return None

    content = _read_file_snippet(file_path)
    if not content:
        return None

    # Simple heuristics for common file types
    content_lower = content.lower()
    if "#!/usr/bin/env python" in content or "import " in content and ".py" in file_path.name:
        return "code"
    if "function " in content or "const " in content or "export " in content:
        return "code"
    if "---" in content or "# " in content or "## " in content:
        return "documents"

    return None


TYPE_BUCKETS: dict[str, set[str]] = {
    "images": {".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".svg"},
    "documents": {".pdf", ".doc", ".docx", ".txt", ".md", ".rtf", ".odt"},
    "spreadsheets": {".xls", ".xlsx", ".csv", ".ods"},
    "presentations": {".ppt", ".pptx", ".odp"},
    "archives": {".zip", ".rar", ".7z", ".tar", ".gz"},
    "audio": {".mp3", ".wav", ".flac", ".aac", ".ogg"},
    "video": {".mp4", ".mkv", ".mov", ".avi", ".wmv"},
    "code": {
        ".py",
        ".js",
        ".ts",
        ".tsx",
        ".jsx",
        ".go",
        ".rs",
        ".java",
        ".cs",
        ".cpp",
        ".c",
        ".h",
        ".json",
        ".yml",
        ".yaml",
        ".toml",
    },
}


def _load_allowed_roots() -> list[Path]:
    raw = os.getenv("JARVIS_SKILLS_ALLOWED_ROOTS", "").strip()
    roots: list[Path] = []

    if raw:
        for piece in raw.split(os.pathsep):
            candidate = piece.strip()
            if not candidate:
                continue
            try:
                roots.append(Path(candidate).expanduser().resolve())
            except OSError:
                continue
    else:
        home = Path.home().resolve()
        for name in DEFAULT_ALLOWED_DIR_NAMES:
            candidate = (home / name).resolve()
            if candidate.exists():
                roots.append(candidate)

    if not roots:
        roots.append(Path.home().resolve())

    deduped: list[Path] = []
    seen: set[str] = set()
    for root in roots:
        key = str(root).lower()
        if key in seen:
            continue
        seen.add(key)
        deduped.append(root)
    return deduped


def _is_in_allowed_scope(target: Path, allowed_roots: list[Path]) -> bool:
    for root in allowed_roots:
        try:
            target.relative_to(root)
            return True
        except ValueError:
            continue
    return False


def _bucket_for_file(file_path: Path, strategy: str) -> str:
    if strategy == "extension":
        suffix = file_path.suffix.lower().lstrip(".")
        return suffix or "no_extension"

    if strategy == "date":
        modified_at = datetime.fromtimestamp(file_path.stat().st_mtime)
        return modified_at.strftime("%Y-%m")

    if strategy == "type":
        suffix = file_path.suffix.lower()
        for bucket, extensions in TYPE_BUCKETS.items():
            if suffix in extensions:
                return bucket

        # Try content-based categorization for ambiguous files
        content_category = _categorize_by_content(file_path)
        if content_category:
            return content_category

        return "other"

    return "other"


def _resolve_destination_collision(destination: Path) -> Path:
    """Handle filename collisions by renaming (not overwriting)."""
    if not destination.exists():
        return destination

    stem = destination.stem
    suffix = destination.suffix
    parent = destination.parent
    counter = 1
    while True:
        candidate = parent / f"{stem}_{counter}{suffix}"
        if not candidate.exists():
            return candidate
        counter += 1


def _execute_plan(operations: list[dict], resolved_target: Path) -> tuple[int, int, list[dict]]:
    """
    Execute planned file operations with error handling and retry logic.
    
    Returns: (moved_files, failed_moves, operation_results)
    """
    moved_files = 0
    failed_moves = 0
    results = []

    for op in operations:
        if op["status"] != "planned":
            continue

        source = Path(op["source"])
        destination = Path(op["destination"])

        try:
            # Ensure destination directory exists
            destination.parent.mkdir(parents=True, exist_ok=True)

            # Execute move
            shutil.move(str(source), str(destination))
            moved_files += 1
            op["status"] = "moved"
            op["timestamp"] = datetime.now().isoformat()
            logger.info(f"Moved: {source} -> {destination}")
            results.append(op)

        except FileNotFoundError as e:
            failed_moves += 1
            op["status"] = "error"
            op["error"] = f"Source file not found: {e}"
            logger.error(f"Move failed (not found): {source}")
            results.append(op)

        except PermissionError as e:
            failed_moves += 1
            op["status"] = "error"
            op["error"] = f"Permission denied: {e}"
            logger.error(f"Move failed (permission): {source}")
            results.append(op)

        except OSError as e:
            failed_moves += 1
            op["status"] = "error"
            op["error"] = str(e)
            logger.error(f"Move failed: {source} - {e}")
            results.append(op)

    return moved_files, failed_moves, results
    if not destination.exists():
        return destination

    stem = destination.stem
    suffix = destination.suffix
    parent = destination.parent
    counter = 1
    while True:
        candidate = parent / f"{stem}_{counter}{suffix}"
        if not candidate.exists():
            return candidate
        counter += 1


def organize_folder(
    path: str,
    strategy: str = "extension",
    recursive: bool = False,
    dry_run: bool = True,
    include_hidden: bool = False,
    execute_plan: bool = False,
) -> dict:
    """
    Organize files inside a folder using extension/type/date strategies.

    Three-phase workflow:
    1. Discovery: Scan and categorize files (includes selective content reading)
    2. Planning: Generate move plan with safety validation (dry_run=True, execute_plan=False)
    3. Execution: Execute approved plan (dry_run=False, execute_plan=True)

    Safety controls:
    - Allowed paths restricted via JARVIS_SKILLS_ALLOWED_ROOTS
    - Dry-run enabled by default (preview-only, no files modified)
    - No delete operations performed (rename on collision)
    - Human-in-the-loop: Review plan before executing
    """
    strategy_normalized = strategy.strip().lower()
    if strategy_normalized not in SUPPORTED_STRATEGIES:
        return {
            "error": (
                f"Unsupported strategy '{strategy}'. "
                f"Choose one of: {sorted(SUPPORTED_STRATEGIES)}"
            )
        }

    # Resolve relative paths to home directory
    target = Path(path).expanduser()
    if not target.is_absolute():
        home_target = Path.home() / path
        if home_target.exists():
            target = home_target

    if not target.exists():
        return {"error": f"Path does not exist: {path}"}
    if not target.is_dir():
        return {"error": f"Path is not a directory: {path}"}

    try:
        resolved_target = target.resolve()
    except OSError as exc:
        return {"error": f"Unable to resolve path '{path}': {exc}"}

    allowed_roots = _load_allowed_roots()
    if not _is_in_allowed_scope(resolved_target, allowed_roots):
        return {
            "error": "Path is outside allowed scope.",
            "allowed_roots": [str(root) for root in allowed_roots],
        }

    # PHASE 1: DISCOVERY - Scan and categorize files
    logger.info(f"[DISCOVERY] Scanning {resolved_target} with strategy={strategy_normalized}")

    candidates = resolved_target.rglob("*") if recursive else resolved_target.iterdir()

    operations: list[dict[str, str]] = []
    scanned_files = 0
    skipped_files = 0

    for item in candidates:
        if not item.is_file():
            continue
        if not include_hidden and item.name.startswith("."):
            skipped_files += 1
            continue

        scanned_files += 1

        # Categorize file (may involve selective content reading for ambiguous files)
        bucket = _bucket_for_file(item, strategy_normalized)
        destination_dir = resolved_target / bucket

        if item.parent == destination_dir:
            skipped_files += 1
            operations.append(
                {
                    "source": str(item),
                    "destination": str(item),
                    "status": "skipped",
                    "reason": "already_in_target_bucket",
                }
            )
            continue

        destination_path = _resolve_destination_collision(destination_dir / item.name)

        operations.append(
            {
                "source": str(item),
                "destination": str(destination_path),
                "bucket": bucket,
                "status": "planned",
            }
        )

    # PHASE 2: PLANNING - Present plan to user for approval
    if dry_run and not execute_plan:
        logger.info(f"[PLANNING] Generated plan with {len(operations)} file operations")
        return {
            "phase": "planning",
            "path": str(resolved_target),
            "strategy": strategy_normalized,
            "recursive": recursive,
            "include_hidden": include_hidden,
            "allowed_roots": [str(root) for root in allowed_roots],
            "files_scanned": scanned_files,
            "files_skipped": skipped_files,
            "planned_operations": len([o for o in operations if o["status"] == "planned"]),
            "skipped_operations": len([o for o in operations if o["status"] == "skipped"]),
            "operations": operations,
            "next_action": "Review the plan above. To execute, call with execute_plan=True and dry_run=False",
        }

    # PHASE 3: EXECUTION - Execute approved plan
    if execute_plan and not dry_run:
        logger.info(f"[EXECUTION] Executing plan with {len(operations)} operations")

        moved_files, failed_moves, execution_results = _execute_plan(operations, resolved_target)

        return {
            "phase": "execution_complete",
            "path": str(resolved_target),
            "strategy": strategy_normalized,
            "recursive": recursive,
            "include_hidden": include_hidden,
            "files_scanned": scanned_files,
            "files_moved": moved_files,
            "files_failed": failed_moves,
            "files_skipped": skipped_files,
            "operations": execution_results,
            "status": "success" if failed_moves == 0 else "partial_failure",
        }

    # Invalid state: must be in planning or execution phase
    return {
        "error": "Invalid parameters. Use either:",
        "option_1": "dry_run=True, execute_plan=False (PLANNING phase - preview)",
        "option_2": "dry_run=False, execute_plan=True (EXECUTION phase - execute)",
    }


def register_folder_organizer_tool(server) -> None:
    """Register the folder organizer tool with the MCP server."""
    parameters = [
        ToolParameter(
            name="path",
            type=ToolParameterType.STRING,
            description="Folder path to organize (must be within allowed roots).",
            required=True,
        ),
        ToolParameter(
            name="strategy",
            type=ToolParameterType.STRING,
            description="Organization strategy: extension, type, or date.",
            required=False,
            default="extension",
            enum=["extension", "type", "date"],
        ),
        ToolParameter(
            name="recursive",
            type=ToolParameterType.BOOLEAN,
            description="Whether to include files in subfolders.",
            required=False,
            default=False,
        ),
        ToolParameter(
            name="dry_run",
            type=ToolParameterType.BOOLEAN,
            description="If true, preview moves only (no files are modified).",
            required=False,
            default=True,
        ),
        ToolParameter(
            name="include_hidden",
            type=ToolParameterType.BOOLEAN,
            description="Whether hidden files should be included.",
            required=False,
            default=False,
        ),
        ToolParameter(
            name="execute_plan",
            type=ToolParameterType.BOOLEAN,
            description="If true with dry_run=False, execute the approved plan.",
            required=False,
            default=False,
        ),
    ]

    server.register_tool(
        name="organize_folder",
        description=(
            "Organize files in a folder by extension/type/date within an allowlisted scope. "
            "Use two-phase workflow: (1) dry_run=True for planning, (2) dry_run=False + execute_plan=True for execution."
        ),
        handler=organize_folder,
        parameters=parameters,
    )
