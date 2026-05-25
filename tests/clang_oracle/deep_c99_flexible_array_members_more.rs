use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn flexible_struct_array_member_pointer_difference_stays_integer_matches_host_stdout_and_exit_code()
{
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int x; int y; } item_t; typedef struct { int length; item_t items[]; } bag_t; int main(void) { bag_t *bag = (bag_t*)malloc(sizeof(bag_t) + 2 * sizeof(item_t)); item_t *cursor; bag->items[0].x = 3; bag->items[0].y = 5; bag->items[1].x = 7; bag->items[1].y = 11; cursor = bag->items + 1; puts(\"flex-struct-diff-int\"); return cursor->x + (cursor - bag->items); }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_struct_array_member_pointer_difference_stays_integer",
        source,
    });
}
