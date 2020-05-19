factorial [n]
    (fact-helper n 1)

fact-helper [n acc]
    (let prod (* n acc)
        (if (= n 1)
            acc
            (fact-helper (- n 1) prod)))

when {a b}
    (list 'if a b :blang)

whenner {a b}
    (when a b)

cool-print {n}
    (list 'print n)

ones {n}
    (if (= n 0)
        (list)
        (cons 1 (ones (- n 1))))

main []
    (print '(ones 4))