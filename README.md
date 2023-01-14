# YAP

Yet Another Password (Manager)

Yap is a password manager meant to be used as a CLI. The goal of this password manager is to *never* actually
see the passwords. Passwords can be generated without the need to copy/paste, and can be copied directly into the 
clipboard. Additionally, the password manager has the ability to sync with a remote repository if desired.

## Commands

The CLI is self-documenting and all instructions can be read from `help` commands.

## Architecture

### Password Storage

Passwords are stored encrypted in a folder in the user's home directory. See possible solutions for details.

This file can be decrypted using a master key, which can be provided with a password. In the case of a password, 
the password is used to derive a key using PBKDF2 and SHA-256 (based on BitWarden's method). 

Possible solutions:

1. Store each password in a separate file. The file name corresponds to the key used, encrypted the same way that the
password itself is encrypted.

2. Store passwords in a single file. This file should first be decrypted, then passwords can be read using the key
given as an argument. This may be more simple to implement, but has the drawback of letting *all* passwords be visible
after decryption.

### Remote Repository Sync
