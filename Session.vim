let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
doautoall SessionLoadPre
silent only
silent tabonly
cd ~/Projects/Programming/xpedition
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
set shortmess+=aoO
badd +65 src/main.rs
badd +400 src/render.rs
badd +286 src/tick.rs
badd +1 src/physics.rs
badd +63 src/entities.rs
badd +10 src/lib.rs
badd +23 term://~/Projects/Programming/xpedition//889321:/usr/bin/bash
badd +32 src/render_entities.rs
badd +62 src/render_pipeline.rs
badd +9018 term://~/Projects/Programming/xpedition//959153:/usr/bin/bash
argglobal
%argdel
$argadd src/main.rs
set stal=2
tabnew +setlocal\ bufhidden=wipe
tabrewind
edit src/render_pipeline.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd _ | wincmd |
split
1wincmd k
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
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe '2resize ' . ((&lines * 17 + 29) / 59)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
exe '3resize ' . ((&lines * 38 + 29) / 59)
exe 'vert 3resize ' . ((&columns * 118 + 118) / 237)
argglobal
balt src/render.rs
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
let s:l = 63 - ((32 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 63
normal! 023|
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Projects/Programming/xpedition//959153:/usr/bin/bash", ":p")) | buffer term://~/Projects/Programming/xpedition//959153:/usr/bin/bash | else | edit term://~/Projects/Programming/xpedition//959153:/usr/bin/bash | endif
if &buftype ==# 'terminal'
  silent file term://~/Projects/Programming/xpedition//959153:/usr/bin/bash
endif
balt ~/Projects/Programming/xpedition/src/render_pipeline.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 9018 - ((16 * winheight(0) + 8) / 17)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 9018
normal! 03|
lcd ~/Projects/Programming/xpedition
wincmd w
argglobal
if bufexists(fnamemodify("~/Projects/Programming/xpedition/src/render.rs", ":p")) | buffer ~/Projects/Programming/xpedition/src/render.rs | else | edit ~/Projects/Programming/xpedition/src/render.rs | endif
if &buftype ==# 'terminal'
  silent file ~/Projects/Programming/xpedition/src/render.rs
endif
balt term://~/Projects/Programming/xpedition//959153:/usr/bin/bash
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
let s:l = 398 - ((15 * winheight(0) + 19) / 38)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 398
normal! 0
lcd ~/Projects/Programming/xpedition
wincmd w
2wincmd w
exe 'vert 1resize ' . ((&columns * 118 + 118) / 237)
exe '2resize ' . ((&lines * 17 + 29) / 59)
exe 'vert 2resize ' . ((&columns * 118 + 118) / 237)
exe '3resize ' . ((&lines * 38 + 29) / 59)
exe 'vert 3resize ' . ((&columns * 118 + 118) / 237)
tabnext
edit ~/Projects/Programming/xpedition/src/entities.rs
argglobal
balt term://~/Projects/Programming/xpedition//889321:/usr/bin/bash
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
let s:l = 63 - ((48 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 63
normal! 022|
lcd ~/Projects/Programming/xpedition
tabnext 1
set stal=1
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
