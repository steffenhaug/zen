import sys

def is_def(s): return s[0] == '@'
def is_ref(s): return s[0] == '!'

def is_bin(s):
    if len(s) < 2: return False
    else: return s[1] == 'b'

def is_byte(x):
    try:
        if is_bin(x):
            int(x, 2)
        else:
            int(x)
        return True
    except:
        return False

def to_int(x):
    if is_bin(x):
        return int(x, 2)
    else:
        return int(x)

def get_op(s, symbols):
    if is_def(s): return 4
    if is_ref(s): return symbols[s[1:]]
    if is_byte(s): return to_int(s)
    return {
           "HALT":                0,
           "LOADI":               1,
           "ADD":                 2,
           "PRINT":               3,
           "NOP":                 4,
           "JUMP":                5,
           "LOADR":               6,
           "JEQ":                 7,
           "COMP":                8,
           "JNEQ":                9,
           "COLOR":              10,
           "INC":                11,
           "DEC":                12,
           "CALL":               13,
           "RET":                14,
           "INPUT":              15,
           "AND":                16,
           "XOR":                17,
           "OR":                 18,
           "SUB":                19,
           "DIV":                20,
           "MUL":                21,
           "JZ":                 22,
           "JNZ":                23,
           "PRINTB":             24,
           "ANDI":               25,
           "COLORI":             26,
           "DRAW":               27,
           "PUSH":               28,
           "POP":                29,
           "JUMPDT":             30,
    }[s]


def compile(infile):
    f = open(infile).read()
    symbols = dict()
    program = bytearray()

    # find all the defs
    for addr, op in enumerate(f.split()):
        print(addr, op)
        if is_def(op):
            symbols[op[1:]] = addr

    for op in f.split():
        program.append(get_op(op, symbols))

    print("main: ", symbols["main"])
    print(symbols)
    return program




if __name__ == "__main__":
    infile  = sys.argv[1]
    outfile = sys.argv[2]
    p = compile(infile)
    with open(outfile, "wb") as f:
        f.write(p)
