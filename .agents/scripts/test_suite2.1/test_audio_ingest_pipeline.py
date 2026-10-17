#!/usr/bin/env python3
"""INGEST SYSTEM test: audio -> transcribe -> ingest pipeline.

Verifies the full pipeline:
1. transcribe_audio on a .wav file
2. ingest the transcribed text result
"""

import asyncio
import tempfile
from pathlib import Path

import pytest


@pytest.mark.asyncio
async def test_audio_transcribe_ingest_pipeline(client):
    """Audio -> transcribe -> ingest pipeline."""
    # 1. Find the test audio file
    audio_path = Path(".agents/scripts/test_suite2.1/test_audio.wav")
    assert audio_path.exists(), f"Test audio not found: {audio_path}"

    # 2. Transcribe audio
    result = await client.call_tool(
        "transcribe_audio",
        {
            "path": str(audio_path.resolve()),
            "store_as_memory": False,
        },
    )
    assert result.get("success") is True, f"Transcription failed: {result}"
    text = result.get("text", "")
    assert len(text) > 0, "Transcription produced empty text"
    print(f"Transcribed text length: {len(text)} chars")

    # 3. Write transcribed text to a temp file for ingestion
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", delete=False, dir=".agents/scripts/test_suite2.1"
    ) as f:
        f.write(text)
        text_path = f.name

    # 4. Ingest the transcribed text
    ingest_result = await client.call_tool(
        "ingest_files",
        {
            "file_path": text_path,
            "memory_type": "file",
        },
    )
    assert ingest_result.get("success") is not False, f"Ingest failed: {ingest_result}"
    print(f"Ingest result: {ingest_result}")
