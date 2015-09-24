import threading
from ctypes import CDLL


def worker(val):
    count = 0
    while count < 15000000:
        count += 1

    print val, count


def main():
    lib = CDLL(
        "/home/ciaran/Documents/rust/web_scraper/"
        "target/debug/libweb_scraper.so"
    )
    print lib.process("http://google.com")
    # threads = []
    # for i in range(10):
    #     t = threading.Thread(target=worker, args=(i,))
    #     threads.append(t)
    #     t.start()

    # for t in threads:
    #     t.join()


if __name__ == "__main__":
    main()
