
# goreutils

The posix utilities are lackluster in presentation. `ls`? `rm`? `mkdir`? Useful, yes, but they have been around for decades. It's finally time, we have space to experiment with our productivity.

The goreutils project aims to bring new and powerful ideas into fruition. One execution at a time.

```bash
# for every file in the current directory, swap 2 random bytes 20 times
px --swap --loop 20 .
# randomize bytes between 10 and 50 on /etc/passwd
sudo px --poke --range 10 50 /etc/passwd

# segmentation fault
seg

# generate a random "word", 5 to 8 characters long
salps --word 5:9

# print arbitrary text
echo "meow" | rat
```

