# k8s
I run this binary on any ARM node on my [cluster][cluster] with a cron job. If
you want to use the [deployment config](cron.yml), then don't forget to change
the version appropriately.

Make sure that the `suckless-hn` namespace exists and following AWS secrets are
added before applying the recipes.

```bash
microk8s.kubectl create secret generic suckless-hn-aws \
    --from-literal=key=xxx \
    --from-literal=secret=xxx \
    -n suckless-hn
```

<!-- References -->
[cluster]: https://github.com/bausano/cluster
