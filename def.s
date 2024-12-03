;.set le

.dreg r0 0
.dreg r1 1
.dreg r2 2
.dreg r3 3

.dins dec 1001010ddddd1010 d=%
.dins add 000011rdddddrrrr r=% d=%
.dins jmp 1001010kkkkk110kkkkkkkkkkkkkkkkk k=$

.org 0
start:
	dec r0
	jmp start


.debug_trap
.panic

.byte
