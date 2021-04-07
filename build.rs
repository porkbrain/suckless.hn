fn main() {
    // don't judge me, I've already had this written in sed
    std::process::Command::new("/bin/sed")
        // edit in place
        .arg("-i")
        // extended regex
        .arg("-r")
        .arg(format!(
            "/{}\\:{}/s//{}:{}/",        // replace "/what/s//with/"
            "porkbrain\\/suckless\\.hn", // regex-escaped
            "[0-9]+\\.[0-9]+\\.[0-9]+\
            (\\-[0-9A-Za-z\\.\\-]+)?", // naive semver
            "porkbrain\\/suckless.hn",   // slash-escaped
            env!("CARGO_PKG_VERSION")
        ))
        .arg("k8s/cron.yml")
        .status()
        .expect("failed to replace version in k8s/cron.yml with sed");
}
