
; =============================================== ;
;                 Collision Test                  ;
;                                                 ;
; This is a visual test of a bug where the carry  ;
; bit is set on sprite collision, even when the   ;
; new pixels are zero.                            ;
;                                                 ;
; When drawing a pixel to a location that already ;
; contains a pixel, the existing pixel must       ;
; be erased. On erase the carry bit (register vf) ;
; must be set to 1.                               ;
;                                                 ;
; This is how Chip8 programs detect collisions    ;
; between sprites.                                ;
;                                                 ;
; The falling ball must break the brick on the    ;
; left. If the brick reappears when the ball      ;
; wraps around and passes by again, then the      ;
; carry bit for erasing pixels is not being set   ;
; correctly.                                      ;
;                                                 ;
; This happens because the ball sprite, like all  ;
; sprites in Chip8, is 8 pixels wide. The zero    ;
; bits on the right are colliding with the brick  ;
; on the right, triggering a redraw of the left   ;
; brick.                                          ;
; =============================================== ;

.variables
    ; ball
    LD  v0, 0   ; x := 0
    LD  v1, 0   ; y := 0
    LD  v2, 0   ; d := 0 ; direction
    ; brick
    LD  v3, 0   ; x := 0
    LD  v4, 0   ; y := 0

; ----------------------------------------------- ;
.main
    ; draw two bricks next to each other
    ;
    ;  brick 1
    ; ┌┴─┐
    ; ########
    ;     └┬─┘
    ;      brick 2
    LD  v3, 16      ; x := 16
    LD  v4, 12      ; y := 12
    LD  I,  .brick
    DRW v3, v4, 1
    ADD v3, 4       ; x += 4
    DRW v3, v4, 1

    ; init ball
    LD  v0, 16      ; x := 16
    LD  v1, 8       ; y := 8
    LD  I,  .ball
    DRW v0, v1, 1

; ----------------------------------------------- ;
.loop
    ; draw ball
    LD  I,  .ball
    DRW v0, v1, 1
    ADD v1, 1       ; y += 1
    DRW v0, v1, 1

    ; collision check
    SE  vf, 1       ; carry == 1
    JP  .loop

    ; collide
    LD  v3, 16      ; x := 16
    LD  v4, 12      ; y := 12
    LD  I,  .brick
    DRW v3, v4, 1

    JP  .loop

; ----------------------------------------------- ;
.ball
    0b10000000
    0b00000000

.brick
    0xF0
    0x00
