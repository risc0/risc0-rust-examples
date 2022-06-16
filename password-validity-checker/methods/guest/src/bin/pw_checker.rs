// Copyright 2022 Risc0, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]
#![no_std]

use risc0_zkvm_guest::{env, sha};

risc0_zkvm_guest::entry!(main);

/* things we could do

convert to Vec<u8> (import some stuff to help, look in the dev chat)
convert str to String.as_bytes()
*/

pub fn main() {
   let password: &str = env::read();
   let salt_bytes: &[u8] = env::read();

   let password_checker = PasswordValidityChecker{
                                             min_length: 3,
                                             max_length: 64,
                                             min_numeric: 2,
                                             min_uppercase: 2,
                                             min_lowercase: 2,
                                             min_special_chars: 1
                                          };

   let valid_password: bool = password_checker.check_password_validity(&password);
   if !valid_password {
        panic!("Password invalid. Please try again.");
   }

   let mut password_byte_vec = password.as_bytes().to_vec();
   let salted_password = password_byte_vec.extend(&salt_bytes.to_vec());
   let password_hash = sha::digest(&salted_password);
    env::commit(&password_hash);
    env::commit(&salt_bytes);
}

pub struct PasswordValidityChecker {
   pub min_length: usize,
   pub max_length: usize,
   pub min_uppercase: usize,
   pub min_lowercase: usize,
   pub min_numeric: usize,
   pub min_special_chars: usize
}

impl PasswordValidityChecker {
   pub fn check_password_validity(&self, pw: &str) -> bool {
      self.correct_length(pw)
      && (self.count_numeric(pw) > self.min_numeric - 1)
      && (self.count_uppercase(pw) > self.min_uppercase - 1)
      && (self.count_lowercase(pw) > self.min_lowercase - 1)
      && (self.count_special_chars(pw) > self.min_special_chars - 1)
   }
   
   fn correct_length(&self, password: &str) -> bool {
      password.len() > (self.min_length - 1) && password.len() < (self.max_length + 1)
   }

   fn count_numeric(&self, password: &str) -> usize {
      let mut numeric_char_count: usize = 0;
      for c in password.chars() {
         if c.is_ascii_digit() {
            numeric_char_count += 1;
         }
      }
      numeric_char_count
   }

   fn count_special_chars(&self, password: &str) -> usize {
      let mut special_char_count: usize = 0;
      for c in password.chars() {
         if c.is_ascii_punctuation() {
            special_char_count += 1;
         }
      }
      special_char_count
   }

   fn count_uppercase(&self, password: &str) -> usize {
      let mut caps_char_count: usize = 0;
      for c in password.chars() {
         if c.is_ascii_uppercase() {
            caps_char_count += 1;
         }
      }
      caps_char_count
   }

   fn count_lowercase(&self, password: &str) -> usize {
      let mut lower_char_count: usize = 0;
      for c in password.chars() {
         if c.is_ascii_lowercase() {
            lower_char_count += 1;
         }
      }
      lower_char_count
   }
}