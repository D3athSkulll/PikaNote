 s 1e10, 20e50, 10.3e5, 
 Visual Separators:
 1_00, 1_000_1, 1_000_000_000
 Literals:
 0x1, 0X2, 0b1, 0B0, 0X10F, 0o1

// Valid/Should be highlighted:
 '1' 'a' 'b' '👍' '\x1b', 'notacharacter' '\'', '\\' '1''2''3' '1'notchar'2'




// Should be highlighted as lifetime specifier:
 'a 'this_is_cool <'abc> '123


// Invalid Integers:

 1e, e3, e, 1e2e, 5.8e10.1
// Invalid Visual Separators:
 _100_1, 100_, 1_00_, _
// Invalid Literals:
 0b102 0x1G, 1o108, 0xxx

// Invalid/ should not be highlighted:
 "a", 'b c'

