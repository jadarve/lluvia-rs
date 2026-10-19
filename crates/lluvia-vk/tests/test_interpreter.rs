#[cfg(test)]
mod tests {

    use lluvia_vk as ll;

    use anyhow::Result;

    #[test]
    fn can_create_new_interpreter() -> Result<()> {
        let _interpreter = ll::Interpreter::new()?;

        Ok(())
    }
}
