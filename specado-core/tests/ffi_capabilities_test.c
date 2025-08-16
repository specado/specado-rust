// Simple C test for capability FFI functions
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// FFI function declarations
extern char* specado_get_openai_manifest(void);
extern char* specado_get_anthropic_manifest(void);
extern void specado_free_string(char* s);

int main() {
    printf("Testing Capability FFI Functions\n");
    printf("================================\n\n");
    
    // Test OpenAI manifest
    printf("1. Getting OpenAI manifest...\n");
    char* openai_json = specado_get_openai_manifest();
    if (openai_json != NULL) {
        // Just check it starts with JSON
        if (strncmp(openai_json, "{\"info\"", 7) == 0) {
            printf("   ✅ OpenAI manifest retrieved (JSON length: %zu)\n", strlen(openai_json));
        } else {
            printf("   ❌ Invalid JSON returned\n");
        }
        specado_free_string(openai_json);
    } else {
        printf("   ❌ NULL returned\n");
    }
    
    // Test Anthropic manifest  
    printf("\n2. Getting Anthropic manifest...\n");
    char* anthropic_json = specado_get_anthropic_manifest();
    if (anthropic_json != NULL) {
        // Just check it starts with JSON
        if (strncmp(anthropic_json, "{\"info\"", 7) == 0) {
            printf("   ✅ Anthropic manifest retrieved (JSON length: %zu)\n", strlen(anthropic_json));
        } else {
            printf("   ❌ Invalid JSON returned\n");
        }
        specado_free_string(anthropic_json);
    } else {
        printf("   ❌ NULL returned\n");
    }
    
    printf("\n✅ FFI test complete!\n");
    return 0;
}