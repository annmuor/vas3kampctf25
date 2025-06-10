### Offline
#### 📍Добро пожаловать
Flag was written on a camp map: `ctf{flag}`
#### 📍Бар
Flag was on a QR-code on a bat boat: `ctf{welcome_to_pub}`
#### 📍Радио
Flag was transmitted few times on a radio: `four monkeys await`
#### 📍Кофейная станция
Flag was on the coffee bag with flag: `freeshka`
#### 📍Почта
Flag was provided by the agent: `ctf{po4ta!}`
#### 📍Горячие Беляши
Flag was provided by the cook: `ctf{a8ldtim_06g5d3bc$rdt}`
#### 📍Кальянная
Flag was on a baseball cap of one of shisha guru: `ctf{vas3k_cap}`
#### В центр неба за победой
Flag was provided based on GO solves: `ctf{天元_ftw}`
#### Embedded: Human-Error processor
Flag was provided by flashing Morse code: `there is no spoon`

### Hidden
#### Секретный флаг
This flag was mentioned in help: `ctf{th1s_1s_fl4g}`
#### Hacker The Brave
Thi flag was in QR code in Bagpipe event pdf: `ctf{hear_the_pipes_are_calling}`
#### Задание на сайте
This flag was given to everyone who visited [main domain](http://vctf.net): `ctf{guess_it_right!}`

### Online
#### Интро
This flag was made by replacing letters to numbers: `ctf{w3lc0m3_t0_c4mp}`
#### Квест: Веб
This flag was on a Quest (by Danis) site in .env file: `ctf{br0k3n_c0ntr0l}`
#### Квест: Бонусное задание
This flag was given by Quest (by Danis) bot command /flag: `ctf{n30n_4rc4d3}`
#### [Lost X](shell1)
Classic task with `chmod -x chmod`. Solution:
```bash
/lib64/ld-linux-x86-64.so.2 /usr/bin/chmod 600 /flag
cat /flag
```
Flag: `ctf{ld_so_is_useful}`

#### [Жизнь без кота](shell2)
Second classic task - read text file in bash without software (cat/more/less/etc).
Solution:
```bash
echo $(</flag)
```
Flag: `ctf{c4t_1s_n0t_n33d3d}`

#### [Кодировка~~](f.txt)
This is actually an ASCII art flag. Just split lines and you'll get the flag
```text
      0000   11111111   000000   11  00000     11   00    00   11111   0000     11  00
     00         11      00       1   00  00         0000  00   11 11   00  0   111   0
    00          11      0000    11   00000     11   00 00 00   11  1   0000     11   00
    00          11      00       1   000000    11   00  0000   11111   0000     11   0
     00         11      00       1   00   0    11   00   000   11  1   00 00    11   0
      000       11      00       11  000000    11   00    00   11  1   00  00   11  00
```
Flag: `ctf{binar1}`
#### [Что нас объединяет](i.jpg)
3D image, answer is: `долор`
#### [Мой новый блог](include)
Classic PHP Include bug, to get the flag just set path to `../../../../flag`
Flag is: `ctf{php_1s_f0r_4ll}`
#### [Танцуй, танцуй!](m.mp3)
Flag is hidden into mp3 file in a form of spectrogram image.
Flag is: `ctf{g00d_ear_m4t3}`
#### [Куда ходят роботы?](robots)
Follow [robots.txt](robots/robots.txt) [twice](robots/secret_31337/robots.txt) to get the flag.
Flag is: `ctf{y0u_4r3_n0t_4_r0b0t}`
#### [Посторонним вход воспрещен](sqli)
Simple SQL Injection. Password is `' OR 1=1;--`
Flag is: `ctf{sqli_st1ll_v4l1d!}`
#### И ты, друг?
Base64-encoded Caesar cipher. Flag is: `ctf{ca3sar_a1nt_s3cur3}`
#### [We ❤️ Rust](../telegram-bot/src/main.rs)
Read source code to find secret command (**/s3cr3t_comm4nd**). Flag is: `ctf{s3cret_m4ssag4_fl4g}`