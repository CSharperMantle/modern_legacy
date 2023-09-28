import struct
from os import path as op

import more_itertools as mit


OP_MAJ_LUT: dict[int, tuple[str, bool]] = {
    0: ("NOP", False),
    1: ("ADD", False),
    2: ("SUB", False),
    3: ("MUL", False),
    4: ("DIV", False),
    5: ("Special!", True),
    6: ("Shift!", True),
    7: ("MOVE", False),
    8: ("LDA", False),
    9: ("LD1", False),
    10: ("LD2", False),
    11: ("LD3", False),
    12: ("LD4", False),
    13: ("LD5", False),
    14: ("LD6", False),
    15: ("LDX", False),
    16: ("LDAN", False),
    17: ("LD1N", False),
    18: ("LD2N", False),
    19: ("LD3N", False),
    20: ("LD4N", False),
    21: ("LD5N", False),
    22: ("LD6N", False),
    23: ("LDXN", False),
    24: ("STA", False),
    25: ("ST1", False),
    26: ("ST2", False),
    27: ("ST3", False),
    28: ("ST4", False),
    29: ("ST5", False),
    30: ("ST6", False),
    31: ("STX", False),
    32: ("STJ", False),
    33: ("STZ", False),
    34: ("JBUS", False),
    35: ("IOC", False),
    36: ("IN", False),
    37: ("OUT", False),
    38: ("JRED", False),
    39: ("Jmp!", True),
    40: ("JA!", True),
    41: ("J1!", True),
    42: ("J2!", True),
    43: ("J3!", True),
    44: ("J4!", True),
    45: ("J5!", True),
    46: ("J6!", True),
    47: ("JX!", True),
    48: ("ModifyA!", True),
    49: ("Modify1!", True),
    50: ("Modify2!", True),
    51: ("Modify3!", True),
    52: ("Modify4!", True),
    53: ("Modify5!", True),
    54: ("Modify6!", True),
    55: ("ModifyX!", True),
    56: ("CMPA", False),
    57: ("CMP1", False),
    58: ("CMP2", False),
    59: ("CMP3", False),
    60: ("CMP4", False),
    61: ("CMP5", False),
    62: ("CMP6", False),
    63: ("CMPX", False),
}
OP_MIN_LUT: dict[int, tuple[str | None, ...]] = {
    5: (
        "NUM",
        "CHAR",
        "HLT",
        None,
        None,
        None,
        None,
        None,
        None,
        "NOT",
        "AND",
        "OR",
        "XOR",
    ),
    6: ("SLA", "SRA", "SLAX", "SRAX", "SLC", "SRC", "SLB", "SRB"),
    39: ("JMP", "JSJ", "JOV", "JNOV", "JL", "JE", "JG", "JGE", "JNE", "JLE"),
    40: ("JAN", "JAZ", "JAP", "JANN", "JANZ", "JANP"),
    41: ("J1N", "J1Z", "J1P", "J1NN", "J1NZ", "J1NP"),
    42: ("J2N", "J2Z", "J2P", "J2NN", "J2NZ", "J2NP"),
    43: ("J3N", "J3Z", "J3P", "J3NN", "J3NZ", "J3NP"),
    44: ("J4N", "J4Z", "J4P", "J4NN", "J4NZ", "J4NP"),
    45: ("J5N", "J5Z", "J5P", "J5NN", "J5NZ", "J5NP"),
    46: ("J6N", "J6Z", "J6P", "J6NN", "J6NZ", "J6NP"),
    47: ("JXN", "JXZ", "JXP", "JXNN", "JXNZ", "JXNP", "JXE", "JXO"),
    48: ("INCA", "DECA", "ENTA", "ENNA"),
    49: ("INC1", "DEC1", "ENT1", "ENN1"),
    50: ("INC2", "DEC2", "ENT2", "ENN2"),
    51: ("INC3", "DEC3", "ENT3", "ENN3"),
    52: ("INC4", "DEC4", "ENT4", "ENN4"),
    53: ("INC5", "DEC5", "ENT5", "ENN5"),
    54: ("INC6", "DEC6", "ENT6", "ENN6"),
    55: ("INCX", "DECX", "ENTX", "ENNX"),
}


def get_op(op: int, field: int) -> str:
    match OP_MAJ_LUT.get(op):
        case None:
            return "???"
        case m:
            op_maj, has_op_min = m
    if has_op_min:
        match OP_MIN_LUT.get(op):
            case None:
                return "???"
            case m:
                return "???" if field >= len(m) or m[field] is None else m[field]
    else:
        return op_maj


def format_instr(addr: int, index: int, field: int, op: str) -> str:
    s_op = op.ljust(8)
    s_addr = f"{addr},".ljust(6)
    s_index = f"{index}".ljust(2)
    s_field = f"{field // 8}:{field % 8}"
    return f"{s_op}{s_addr}{s_index}({s_field})"


WD = "./modern_legacy_884f7f66d2d43d5795908425b7f45111675c1fce/"

with open(op.join(WD, "vm_mem.bin"), "rb") as f:
    vm_mem = f.read()

assert len(vm_mem) % 6 == 0

for iw, w in enumerate(mit.ichunked(vm_mem, 6)):
    word = bytes(w)
    sign, addr, index, field, op = struct.unpack(">BhBBB", word)
    if sign != 0:
        addr = -addr
    print(
        f"{str(iw).ljust(4)}\t{word.hex(' ')}\t{format_instr(addr, index, field, get_op(op, field))}"
    )
