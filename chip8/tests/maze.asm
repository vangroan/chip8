
; ============== ;
; maze generator ;
; ============== ;

.main
    LD      v0,  0       ; x := 0
    LD      v1,  0       ; y := 0

.loop
    LD      I,   .right  ; sprite
    RAND    v2,  1       ; v2 := random [0, 1]
    SE      v2,  1       ; if v2 == 1
    LD      I,   .left   ; sprite
    DRW     v0,  v1,  4  ; draw 4 rows at (v0, v1)

    ; next column
    ADD     v0,  4       ; x += 4
    SE      v0,  64      ; if x == 64 (right of display)
    JP      .loop        ; else: again
    LD      v0,  0       ; then: x := 0
    ADD     v1,  4       ; y += 4
    SE      v1,  32      ; if y == 32 (bottom of display)
    JP      .loop        ; else: again

.forever
    JP      .forever     ; then: forever

.left
    0b10000000
    0b01000000
    0b00100000
    0b00010000

.right
    0b00100000
    0b01000000
    0b10000000
    0b00010000
