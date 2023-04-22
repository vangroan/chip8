
; =============================================== ;
;                  Font Set Test                  ;
;                                                 ;
; Test of the Fx29 (LD F, Vx) opcode, which loads ;
; a font sprite address into I.                   ;
; ================================================;


.registers
    LD  va, 0    ; x := 0
    LD  vb, 0    ; y := 0
    LD  vc, 0x0  ; char  := 0
    LD  vd, 255  ; delay := 255
    ; clock digits
    LD  v0, 0
    LD  v1, 0
    LD  v2, 0

; -------------------------------------------------
; print whole font set
.loop
    LD  F,  vc
    DRW va, vb, 5

    ADD vc, 1    ; char += 1
    ADD va, 5    ; x += 4

    SNE vc, 0xa  ; next row
    JP  .next_row

    SE  vc, 0x10 ; if c == 0x10: break
    JP  .loop

    JP  .clock

.next_row
    LD  va, 0    ; x := 0
    ADD vb, 6    ; y += 6
    JP  .loop

; -------------------------------------------------
; clock counter
.clock
    LD  vb, 19  ; y := 20
    LD  vc, 0   ; char := 0

    CALL .clock_draw

.clock_loop
    LD   vd, DT
    SE   vd, 0
    JP   .clock_loop

    CALL .clock_draw  ; erase
    ADD  vc, 1  ; increment clock

    LD   vd, 255
    LD   DT, vd

    CALL .clock_draw
    JP   .clock_loop

.clock_draw
    LD  va, 25  ; x := 25

    LD  I,    .clock_data
    LD  BCD,  vc
    LD  v2,   [I]

    ; 100
    LD  F,  v0
    DRW va, vb, 5

    ; 10
    ADD va, 5
    LD  F,  v1
    DRW va, vb, 5

    ; 1
    ADD va, 5
    LD  F,  v2
    DRW va, vb, 5

    RET

; -------------------------------------------------
.forever
    JP  .forever

; -------------------------------------------------
.clock_data
;    100   10    1
    0x01 0x02 0x03
