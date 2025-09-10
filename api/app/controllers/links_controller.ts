import type { HttpContext } from '@adonisjs/core/http'
import Link from '#models/link'
import { randomBytes } from 'node:crypto'

export default class LinksController {
  private createShortUrl(): string {
    return randomBytes(3).toString('hex')
  }

  private isValidSlug(slug: string): boolean {
    return /^[a-zA-Z0-9_-]+$/.test(slug)
  }

  async shorten({ request, response }: HttpContext) {
    const { url, slug } = request.only(['url', 'slug'])

    if (!url) {
      return response.badRequest({ error: 'URL is required' })
    }

    if (!url.startsWith('gurt://')) {
      return response.badRequest({ error: 'Only gurt:// URLs are supported' })
    }

    let shortUrl: string

    if (slug) {
      if (!this.isValidSlug(slug)) {
        return response.badRequest({
          error: 'Slug can only contain letters, numbers, hyphens, and underscores',
        })
      }

      const existingLink = await Link.findBy('shortUrl', slug)
      if (existingLink) {
        return response.conflict({ error: 'Slug is already taken' })
      }

      shortUrl = slug
    } else {
      shortUrl = this.createShortUrl()
      while (await Link.findBy('shortUrl', shortUrl)) {
        shortUrl = this.createShortUrl()
      }
    }

    const link = await Link.create({
      url,
      shortUrl,
    })

    return response.created({
      user: {
        short_url: link.shortUrl,
      },
    })
  }

  async redirect({ params, response }: HttpContext) {
    const { shortUrl } = params

    const link = await Link.findBy('shortUrl', shortUrl)
    if (!link) {
      return response.notFound({ error: 'Short URL not found' })
    }

    link.clickCount += 1
    await link.save()

    return response.ok({
      url: link.url,
    })
  }
}
