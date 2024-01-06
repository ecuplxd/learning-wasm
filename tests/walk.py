import os


ROOT = os.getcwd()
OUTPUT = "output"
TESTSUITE = "testsuite"
WAST_EXT = ".wast"
EXE = os.path.join(ROOT, "wast2json.exe")


def norm_join(p1, p2):
    return os.path.normpath(os.path.join(p1, p2))


def walk(parent):
    for item in os.listdir(parent):
        cur = norm_join(parent, item)

        if os.path.isdir(cur):
            pass
        elif item.endswith(WAST_EXT):
            file = norm_join(parent, item)
            name = item.replace(WAST_EXT, "").replace("-", "_")
            output = norm_join(parent, name).replace(TESTSUITE, OUTPUT)

            if not os.path.exists(output):
                os.makedirs(output)

            command = " ".join(
                [
                    EXE,
                    "--enable-all",
                    "--debug-names",
                    "--no-check",
                    file,
                    "-o",
                    norm_join(output, name + ".json"),
                ]
            )

            os.system(command)


def main():
    walk(norm_join(ROOT, TESTSUITE))


if __name__ == "__main__":
    main()
