# EncryptedFS
An encrypted file system in Rust that mounts with FUSE. It can be used to create encrypted directories.

# Usage

To use the encrypted file system, you need to have FUSE installed on your system. You can install it by running the following command (or based on your distribution):

```bash
sudo apt-get update
sudo apt-get -y install fuse3
```
A basic example of how to use the encrypted file system is shown below:

```
encrypted_fs --mount-point MOUNT_POINT --data-dir DATA_DIR
```
Where `MOUNT_POINT` is the directory where the encrypted file system will be mounted and `DATA_DIR` is the directory where the encrypted data will be stored.\
It will prompt you to enter a password to encrypt/decrypt the data.

## Change Password
The encryption key is stored in a file and encrypted with a key derived from the password.
This offers the possibility to change the password without needing to decrypt and re-encrypt the whole data.
This is done by decrypting the key with the old password and re-encrypting it with the new password.

To change the password, you can run the following command:
```
encrypted_fs --change-password --data-dir DATA_DIR
```
Where `DATA_DIR` is the directory where the encrypted data is stored.\
It will prompt you to enter the old password and then the new password.

## Encryption info
You can specify the encryption algorithm and derive key hash rounds adding these arguments to the command line:

```
--cipher CIPHER --derive-key-hash-rounds ROUNDS
```
Where `CIPHER` is the encryption algorithm and `ROUNDS` is the number of rounds to derive the key hash.\
You can check the available ciphers with `encrypted_fs --help`.

Default values are `ChaCha20` and `600_000` respectively.
