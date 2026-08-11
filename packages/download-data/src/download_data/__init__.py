from huggingface_hub import snapshot_download


def download_data() -> None:
    snapshot_download(
        "CohereLabs/msmarco-v2.1-embed-english-v3",
        repo_type="dataset",
        allow_patterns=[
            "passages_parquet/msmarco_v2.1_doc_segmented_00.parquet",
        ],
    )


def main() -> None:
    download_data()
