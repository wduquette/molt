proc pexpr {expr} {
    set out [expr $expr]
    puts "$expr == $out"
}

pexpr 12/10
pexpr 12%10
pexpr -12/10
pexpr -12%10
pexpr 12/-10
pexpr 12%-10
pexpr -12/-10
pexpr -12%-10
