import uuid
from argparse import ArgumentParser


def gen_name(prefix: str, suffix: str) -> str:
    return f"{prefix}{uuid.uuid4().hex[:8]}{suffix}"


parser = ArgumentParser()
parser.add_argument("--prefix", "-p", type=str, default="_fun_")
parser.add_argument("--suffix", "-s", type=str, default="")
parser.add_argument("--count", type=int, default=1)
args = parser.parse_args()

for _ in range(args.count):
    print(gen_name(args.prefix, args.suffix))
