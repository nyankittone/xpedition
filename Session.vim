let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
doautoall SessionLoadPre
silent only
silent tabonly
cd ~
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
set shortmess+=aoO
badd +27 term://~//3746:/usr/bin/bash
badd +74 Programming/xpedition/src/main.rs
badd +35 Documents/Notice/notice.html
badd +88 .local/share/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/x11rb-protocol-0.13.2/src/protocol/xproto.rs
badd +6 Programming/xpedition/src/lib.rs
badd +31 Programming/xpedition/game_vec.rs
badd +33 Programming/xpedition/src/game_vector.rs
badd +842 term://~/Programming/xpedition//98378:/usr/bin/bash
badd +4 Programming/xpedition/src/physics.rs
badd +49 .local/share/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/xkeysym-0.2.1/src/lib.rs
badd +32 Programming/xpedition/src/log.rs
badd +24 Programming/xpedition/src/display.rs
badd +328 Programming/xpedition/src/render.rs
argglobal
%argdel
edit Programming/xpedition/src/render.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 117 + 118) / 236)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 236)
argglobal
balt Programming/xpedition/src/main.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 314 - ((13 * winheight(0) + 28) / 57)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 314
normal! 015|
lcd ~/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Programming/xpedition//98378:/usr/bin/bash", ":p")) | buffer term://~/Programming/xpedition//98378:/usr/bin/bash | else | edit term://~/Programming/xpedition//98378:/usr/bin/bash | endif
if &buftype ==# 'terminal'
  silent file term://~/Programming/xpedition//98378:/usr/bin/bash
endif
balt ~/Programming/xpedition/src/main.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 842 - ((56 * winheight(0) + 28) / 57)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 842
normal! 03|
lcd ~/Programming/xpedition
wincmd w
exe 'vert 1resize ' . ((&columns * 117 + 118) / 236)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 236)
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
