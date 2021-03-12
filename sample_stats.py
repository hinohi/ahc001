from pathlib import Path


def main():
    scores = {}
    for line in Path('scores.txt').open():
        seed, score = line.split()
        scores[int(seed)] = score
    for seed in sorted(scores):
        with (Path('tools/in') / f'{seed:04}.txt').open() as f:
            n = int(f.readline())
            s1 = 0
            s2 = 0
            for line in f:
                _, _, size = line.split()
                s1 += int(size)
                s2 += int(size) ** 2
            s1 /= n
            s2 /= n
        print(seed, scores[seed], n, s2 - s1 * s1)


if __name__ == '__main__':
    main()
