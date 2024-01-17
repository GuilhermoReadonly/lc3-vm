#include<stdio.h>


int main(void) {
    int i = 0;
    printf("Starting computation...\n");


    i = recursive(5000);

    printf("Computation done : %d\n", i);
    return 0;
}

int recursive(int i){
    if(i==0){
        return 0;
    }
    else{
        return recursive(i - 1) + 1;
    }

}
