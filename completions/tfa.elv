
set edit:completion:arg-completer[tfa] = { |@words|
    var index = (count $words)
    set index = (- $index 1)

    put (env _CLAP_IFS="\n" _CLAP_COMPLETE_INDEX=(to-string $index) COMPLETE="elvish" tfa -- $@words) | to-lines
}

