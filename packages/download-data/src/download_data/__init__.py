from huggingface_hub import snapshot_download


def download_data() -> None:
    snapshot_download(
        "ashraq/cohere-wiki-embedding-100k",
        repo_type="dataset",
        allow_patterns=[
            "data/train-00000-of-00002-039513d189a50a66.parquet",
            "data/train-00001-of-00002-34bc764abe7090be.parquet",
        ],
    )


def main() -> None:
    download_data()
