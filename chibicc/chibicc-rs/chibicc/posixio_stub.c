#define __RISC0_SYS_HALT 0
int read() { return 0; }
int stderr = 2;
int stdout = 1;
int stdin = 0;
int write() { return 0; }
int lseek() { return 0; }
int close() { return 0; }
int open() { return 0; }
int kill() { return 0; }
int getpid() { return 1000; }
inline void __attribute__((always_inline)) __attribute__((noreturn)) _exit(int code) {
    __asm__ volatile("li t0, %0\t\n"
                     "ecall"
                     : /* no output required */
                     :  "I"(__RISC0_SYS_HALT)
                     :
                     ); 
   while(1); // So that function is properly noreturn
}
int open_memstream() { return 0; }
int stat() { return 0; }
int fmemopen() { return 0; }
