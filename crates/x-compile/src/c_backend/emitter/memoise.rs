use super::*;
use x_diagnostics::backend_messages;

pub(super) const SUPPORT: &str = r#"extern void *memcpy(void*,const void*,unsigned long long);
static void _x_lock(volatile unsigned char*p){unsigned char expected;do{expected=0;__asm__ volatile("lock cmpxchgb %2,%1":"+a"(expected),"+m"(*p):"q"((unsigned char)1):"memory","cc");}while(expected);}
static void _x_unlock(volatile unsigned char*p){__asm__ volatile("":::"memory");*p=0;__asm__ volatile("":::"memory");}
"#;
pub(super) fn capacity(body: &MirBody) -> Result<u32> {
    let character = match &body.call_contract {
        Some(CallContract::Memoise(character)) => character.clone(),
        None => MemoiseCallContract::checked(DEFAULT_MEMOISE_CAPACITY),
    };

    if character.capacity == 0 || character != MemoiseCallContract::checked(character.capacity) {
        return Err(backend_messages::c_emission_unsupported_memoise_contract().into());
    }

    if character.capacity > 1048576 {
        return Err(backend_messages::c_emission_unbounded_memoise_capacity().into());
    }

    Ok(character.capacity)
}

pub(super) fn globals(symbol: &str, signature: &Signature<'_>) -> Result<String> {
    let body = signature.body;
    let count = capacity(body)?;

    if body.is_throws
        || (!matches!(signature.return_value, Type::F32 | Type::F64)
            && width(signature.return_value).is_none())
    {
        return Err(backend_messages::c_emission_nonscalar_or_throwing_memoise_result().into());
    }

    for type_value in signature.parameters {
        if !matches!(type_value, Type::F32 | Type::F64) && width(type_value).is_none() {
            return Err(backend_messages::c_emission_nonscalar_memoise_key().into());
        }
    }

    let mut text = format!(
        "static volatile unsigned char {symbol}_lock;static unsigned char {symbol}_valid[{count}];static {} {symbol}_values[{count}];\n",
        ctype(signature.return_value)?
    );

    for index in 0..signature.parameters.len() {
        writeln!(
            text,
            "static unsigned long long {symbol}_key{index}[{count}];"
        )?;
    }

    Ok(text)
}

fn keys(signature: &Signature<'_>, prefix: &str) -> Result<String> {
    let mut output = String::new();

    for (index, type_value) in signature.parameters.iter().enumerate() {
        if matches!(type_value, Type::F32 | Type::F64) {
            writeln!(
                output,
                "unsigned long long {prefix}_key{index}=0;memcpy(&{prefix}_key{index},&v{},sizeof(v{}));",
                index + 1,
                index + 1
            )?;
        } else {
            writeln!(
                output,
                "unsigned long long {prefix}_key{index}=(unsigned long long)v{};",
                index + 1
            )?;
        }
    }

    writeln!(
        output,
        "unsigned long long {prefix}_hash=14695981039346656037ULL;"
    )?;
    for index in 0..signature.parameters.len() {
        writeln!(
            output,
            "{prefix}_hash=({prefix}_hash^{prefix}_key{index})*1099511628211ULL;"
        )?;
    }

    writeln!(
        output,
        "{prefix}_hash^={prefix}_hash>>33;{prefix}_hash*=18397679294719823053ULL;{prefix}_hash^={prefix}_hash>>33;{prefix}_hash*=14181476777654086739ULL;{prefix}_hash^={prefix}_hash>>33;unsigned long long {prefix}_index={prefix}_hash%{}ULL;",
        capacity(signature.body)?
    )?;

    Ok(output)
}

pub(super) fn lookup(signature: &Signature<'_>) -> Result<String> {
    let symbol = &signature.symbol;
    let mut output = String::from("{\n");
    output.push_str(&keys(signature, "_x_entry")?);
    writeln!(
        output,
        "_x_lock(&{symbol}_lock);unsigned char _x_hit={symbol}_valid[_x_entry_index];"
    )?;
    for index in 0..signature.parameters.len() {
        writeln!(
            output,
            "_x_hit=_x_hit&&({symbol}_key{index}[_x_entry_index]==_x_entry_key{index});"
        )?;
    }

    writeln!(
        output,
        "if(_x_hit){{{} _x_value={symbol}_values[_x_entry_index];_x_unlock(&{symbol}_lock);return _x_value;}}_x_unlock(&{symbol}_lock);}}",
        ctype(signature.return_value)?
    )?;

    Ok(output)
}

pub(super) fn store(signature: &Signature<'_>) -> Result<String> {
    let symbol = &signature.symbol;
    let mut output = String::from("{\n");
    output.push_str(&keys(signature, "_x_store")?);
    writeln!(output, "_x_lock(&{symbol}_lock);")?;
    for index in 0..signature.parameters.len() {
        writeln!(
            output,
            "{symbol}_key{index}[_x_store_index]=_x_store_key{index};"
        )?;
    }

    writeln!(
        output,
        "{symbol}_values[_x_store_index]=v{};{symbol}_valid[_x_store_index]=1;_x_unlock(&{symbol}_lock);}}",
        signature.body.return_local.0
    )?;

    Ok(output)
}
