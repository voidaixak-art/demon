import sys
import argparse

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from subconscious.mind_loop import LivingSubconscious

def main():
    parser = argparse.ArgumentParser(
        description="Axiom Engine 2.0: The Living Subconscious Mind"
    )
    parser.add_argument(
        "--study",
        type=str,
        help="Instruct the subconscious to deep-study a specific domain or subject",
    )
    parser.add_argument(
        "--sleep",
        action="store_true",
        help="Trigger an immediate offline sleep & memory consolidation cycle",
    )
    args = parser.parse_args()

    mind = LivingSubconscious()

    if not mind.check_daemon_connection():
        print("\n❌ [ERROR] Could not connect to Axiom Core daemon at http://localhost:8080.")
        print("💡 Please start the Rust Core first in a separate terminal:")
        print("   cargo run -- daemon\n")
        sys.exit(1)

    if args.study:
        mind.scholar.deep_learn_subject(args.study)
    elif args.sleep:
        mind.run_sleep_and_dream_cycle()
    else:
        mind.start_infinite_loop()

if __name__ == "__main__":
    main()
