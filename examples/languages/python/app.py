from pathlib import Path

from southpaw import require_posture


def main() -> None:
    policy_file = Path(__file__).with_name("southpaw.yaml")
    report = require_posture(policy_file=str(policy_file))
    print({"demo": "python", "status": report["status"]})


if __name__ == "__main__":
    main()
