import ruamel.yaml
import os
import lmdb
import json
import datetime


def parse_yaml_from_file(file_path: str):
    """
    Parses a YAML file and returns the corresponding Python data structure.

    Args:
        file_path: The path to the YAML file.

    Returns:
        A Python dictionary or list representing the parsed YAML,
        or None if parsing fails or the file cannot be read.
    """
    # Initialize the YAML parser.
    # typ='safe' is crucial for security as it prevents arbitrary code execution
    # from untrusted YAML.
    # pure=True forces the Python implementation.
    yaml_parser = ruamel.yaml.YAML(typ="safe", pure=True)

    try:
        with open(file_path, "r", encoding="utf-8") as f:
            # The load() method parses the YAML from the file stream.
            data = yaml_parser.load(f)
        return data
    except FileNotFoundError:
        print(f"Error: File not found at path: {file_path}")
        return None
    except ruamel.yaml.YAMLError as e:
        print(f"Error parsing YAML content from file: {file_path}")
        if hasattr(e, "problem_mark"):
            mark = e.problem_mark
            print(f"  Error is near line {mark.line + 1}, column {mark.column + 1}")
        if hasattr(e, "problem"):
            print(f"  Problem: {e.problem}")
        if hasattr(e, "context_mark") and e.context_mark:
            cmark = e.context_mark
            print(f"  Context is near line {cmark.line + 1}, column {cmark.column + 1}")
        if hasattr(e, "context"):
            print(f"  Context: {e.context}")
        return None  # Or raise the exception: raise
    except Exception as e:
        print(f"An unexpected error occurred while reading or parsing the file: {e}")
        return None


def json_default(obj):
    if isinstance(obj, (datetime.datetime, datetime.date, datetime.time)):
        return obj.isoformat()
    return str(obj)


# --- How to use the parser ---
if __name__ == "__main__":
    conversations_dir = "conversations"
    yaml_files = [f for f in os.listdir(conversations_dir) if f.endswith(".yaml")]
    if not yaml_files:
        print(f"No YAML files found in {conversations_dir}")

    # Open (or create) LMDB environment
    lmdb_env = lmdb.open("conversations.lmdb", map_size=10**9)
    with lmdb_env.begin(write=True) as txn:
        for yaml_file in yaml_files:
            yaml_file_path = os.path.join(conversations_dir, yaml_file)
            parsed_data = parse_yaml_from_file(yaml_file_path)
            print("\n--- Processing file: {} ---".format(yaml_file))
            if parsed_data:
                # Save to LMDB: key = file name without .yaml, value = JSON string
                key = os.path.splitext(yaml_file)[0].encode("utf-8")
                try:
                    value = json.dumps(parsed_data, default=json_default).encode("utf-8")
                except Exception as e:
                    print(f"Failed to serialize {yaml_file} to JSON: {e}")
                    continue
                txn.put(key, value)
                print(
                    "Saved parsed data to LMDB with key: {}".format(key.decode("utf-8"))
                )
                # (Optional) Print summary as before
                if isinstance(parsed_data, dict):
                    print(f"ID: {parsed_data.get('id')}")
                    messages = parsed_data.get("messages", [])
                    if messages and isinstance(messages, list) and len(messages) > 0:
                        first_message = messages[0]
                        if isinstance(first_message, dict):
                            print(
                                "Role of first message: {}".format(
                                    first_message.get("role")
                                )
                            )
                            if first_message.get("role") == "user" and isinstance(
                                first_message.get("content"), list
                            ):
                                content_item = first_message["content"][0]
                                if (
                                    isinstance(content_item, dict)
                                    and "text" in content_item
                                ):
                                    print(
                                        "Text of first user message content: {}".format(
                                            content_item["text"]
                                        )
                                    )
                    tool_result_content_found = False
                    for message in parsed_data.get("messages", []):
                        if message.get("role") == "user" and message.get("content"):
                            for content_item in message["content"]:
                                if (
                                    isinstance(content_item, dict)
                                    and content_item.get("type") == "tool_result"
                                ):
                                    run_info = content_item.get("run", {})
                                    result_info = run_info.get("result", {})
                                    file_content = None
                                    if isinstance(result_info, dict):
                                        file_content = result_info.get("content")
                                    elif isinstance(result_info, list):
                                        print("result_info is a list; skipping content extraction.")
                                    if file_content:
                                        print(
                                            "\n--- Found tool_result content (first 5 lines of TODO.md): ---"
                                        )
                                        print(
                                            "\n".join(file_content.splitlines()[:5])
                                        )
                                        print(
                                            "---------------------------------------------------------"
                                        )
                                        tool_result_content_found = True
                                        break
                            if tool_result_content_found:
                                break
            else:
                print(f"Failed to parse YAML from file: {yaml_file_path}")
